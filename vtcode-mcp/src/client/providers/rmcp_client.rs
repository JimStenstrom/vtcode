//! Low-level RMCP transport client adapter.

use super::handler::LoggingClientHandler;
use anyhow::{Context, Result, anyhow};
use futures::FutureExt;
use mcp_types::{
    CallToolRequestParams, CallToolResult, GetPromptRequestParams,
    GetPromptResult, InitializeRequestParams, InitializeResult, Prompt, ReadResourceRequestParams,
    ReadResourceResult, Resource, Tool,
};
use reqwest::header::HeaderMap;
use rmcp::service::{self, RoleClient, RunningService};
use rmcp::transport::child_process::TokioChildProcess;
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
};
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time;
use tracing::{info, warn};

use crate::client::McpElicitationHandler;

/// Lightweight adapter around the rmcp transport mirroring Codex' `RmcpClient` API.
pub struct RmcpClient {
    provider_name: String,
    state: Mutex<ClientState>,
    elicitation_handler: Option<Arc<dyn McpElicitationHandler>>,
}

enum ClientState {
    Connecting {
        transport: Option<PendingTransport>,
    },
    Ready {
        service: Arc<RunningService<RoleClient, LoggingClientHandler>>,
    },
    Stopped,
}

enum PendingTransport {
    ChildProcess(TokioChildProcess),
    StreamableHttp(StreamableHttpClientTransport<reqwest::Client>),
}

impl RmcpClient {
    pub async fn new_stdio_client(
        provider_name: String,
        program: OsString,
        args: Vec<OsString>,
        working_dir: Option<PathBuf>,
        env: Option<HashMap<String, String>>,
        elicitation_handler: Option<Arc<dyn McpElicitationHandler>>,
    ) -> Result<Self> {
        let mut command = Command::new(&program);
        command
            .kill_on_drop(true)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .env_clear()
            .envs(create_env_for_mcp_server(env));

        if let Some(dir) = working_dir.as_ref() {
            command.current_dir(dir);
        }

        command.args(&args);

        let builder = TokioChildProcess::builder(command);
        let (transport, stderr) = builder.stderr(std::process::Stdio::piped()).spawn()?;

        if let Some(stderr) = stderr {
            let program_name = program.to_string_lossy().into_owned();
            let provider_label = provider_name.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                loop {
                    match reader.next_line().await {
                        Ok(Some(line)) => {
                            info!(
                                provider = provider_label.as_str(),
                                program = program_name.as_str(),
                                message = line.as_str(),
                                "MCP server stderr"
                            );
                        }
                        Ok(None) => break,
                        Err(error) => {
                            warn!(
                                provider = provider_label.as_str(),
                                program = program_name.as_str(),
                                error = %error,
                                "Failed to read MCP server stderr"
                            );
                            break;
                        }
                    }
                }
            });
        }

        Ok(Self {
            provider_name,
            state: Mutex::new(ClientState::Connecting {
                transport: Some(PendingTransport::ChildProcess(transport)),
            }),
            elicitation_handler,
        })
    }

    pub async fn new_streamable_http_client(
        provider_name: String,
        url: &str,
        bearer_token: Option<String>,
        headers: HeaderMap,
        elicitation_handler: Option<Arc<dyn McpElicitationHandler>>,
    ) -> Result<Self> {
        let mut config = StreamableHttpClientTransportConfig::with_uri(url.to_string());
        if let Some(token) = bearer_token {
            config = config.auth_header(token);
        }

        info!(
            "Connecting to MCP HTTP provider '{}' at {}",
            provider_name, url
        );

        let mut client_builder = reqwest::Client::builder();
        if !headers.is_empty() {
            client_builder = client_builder.default_headers(headers);
        }

        let http_client = client_builder.build().with_context(|| {
            format!(
                "failed to construct reqwest client for MCP provider '{}'",
                provider_name
            )
        })?;

        let transport = StreamableHttpClientTransport::with_client(http_client, config);
        Ok(Self {
            provider_name,
            state: Mutex::new(ClientState::Connecting {
                transport: Some(PendingTransport::StreamableHttp(transport)),
            }),
            elicitation_handler,
        })
    }

    pub async fn initialize(
        &self,
        params: InitializeRequestParams,
        timeout: Option<Duration>,
    ) -> Result<InitializeResult> {
        let handler = LoggingClientHandler::new(
            self.provider_name.clone(),
            params,
            self.elicitation_handler.clone(),
        );

        let (transport_future, service_label) = {
            let mut guard = self.state.lock().await;
            match &mut *guard {
                ClientState::Connecting { transport } => match transport.take() {
                    Some(PendingTransport::ChildProcess(transport)) => (
                        service::serve_client(handler.clone(), transport).boxed(),
                        "stdio",
                    ),
                    Some(PendingTransport::StreamableHttp(transport)) => (
                        service::serve_client(handler.clone(), transport).boxed(),
                        "http",
                    ),
                    None => {
                        return Err(anyhow!(
                            "MCP client for {} already initializing",
                            handler.provider_name()
                        ));
                    }
                },
                ClientState::Ready { .. } => {
                    return Err(anyhow!(
                        "MCP client for {} already initialized",
                        handler.provider_name()
                    ));
                }
                ClientState::Stopped => return Err(anyhow!("MCP client has been shut down")),
            }
        };

        let service = match timeout {
            Some(duration) => time::timeout(duration, transport_future)
                .await
                .with_context(|| {
                    format!("Timed out establishing {service_label} MCP transport")
                })??,
            None => transport_future.await?,
        };

        let initialize_result_rmcp = service
            .peer()
            .peer_info()
            .ok_or_else(|| anyhow!("Handshake succeeded but server info missing"))?;
        let initialize_result = convert_to_mcp(initialize_result_rmcp)?;

        let mut guard = self.state.lock().await;
        *guard = ClientState::Ready {
            service: Arc::new(service),
        };

        Ok(initialize_result)
    }

    pub async fn list_all_tools(&self, timeout: Option<Duration>) -> Result<Vec<Tool>> {
        let service = self.service().await?;
        let rmcp_future = service.peer().list_all_tools();
        let rmcp_tools = run_with_timeout(rmcp_future, timeout, "tools/list").await?;

        rmcp_tools
            .into_iter()
            .map(|tool| convert_to_mcp::<_, Tool>(tool))
            .collect::<Result<Vec<_>>>()
            .context("Failed to convert MCP tool list")
    }

    pub async fn list_all_prompts(&self, timeout: Option<Duration>) -> Result<Vec<Prompt>> {
        let service = self.service().await?;
        let rmcp_future = service.peer().list_all_prompts();
        let rmcp_prompts = run_with_timeout(rmcp_future, timeout, "prompts/list").await?;

        rmcp_prompts
            .into_iter()
            .map(|prompt| convert_to_mcp::<_, Prompt>(prompt))
            .collect::<Result<Vec<_>>>()
            .context("Failed to convert MCP prompt list")
    }

    pub async fn list_all_resources(&self, timeout: Option<Duration>) -> Result<Vec<Resource>> {
        let service = self.service().await?;
        let rmcp_future = service.peer().list_all_resources();
        let rmcp_resources = run_with_timeout(rmcp_future, timeout, "resources/list").await?;

        rmcp_resources
            .into_iter()
            .map(|resource| convert_to_mcp::<_, Resource>(resource))
            .collect::<Result<Vec<_>>>()
            .context("Failed to convert MCP resource list")
    }

    pub async fn call_tool(
        &self,
        params: CallToolRequestParams,
        timeout: Option<Duration>,
    ) -> Result<CallToolResult> {
        let service = self.service().await?;
        let rmcp_params: rmcp::model::CallToolRequestParam = convert_to_rmcp(params)?;
        let rmcp_result =
            run_with_timeout(service.call_tool(rmcp_params), timeout, "tools/call").await?;
        convert_call_tool_result(rmcp_result)
    }

    pub async fn read_resource(
        &self,
        params: ReadResourceRequestParams,
        timeout: Option<Duration>,
    ) -> Result<ReadResourceResult> {
        let service = self.service().await?;
        let rmcp_params: rmcp::model::ReadResourceRequestParam = convert_to_rmcp(params)?;
        let rmcp_result = run_with_timeout(
            service.peer().read_resource(rmcp_params),
            timeout,
            "resources/read",
        )
        .await?;
        convert_to_mcp(rmcp_result).context("Failed to convert MCP resource contents")
    }

    pub async fn get_prompt(
        &self,
        params: GetPromptRequestParams,
        timeout: Option<Duration>,
    ) -> Result<GetPromptResult> {
        let service = self.service().await?;
        let rmcp_params: rmcp::model::GetPromptRequestParam = convert_to_rmcp(params)?;
        let rmcp_result = run_with_timeout(
            service.peer().get_prompt(rmcp_params),
            timeout,
            "prompts/get",
        )
        .await?;
        convert_to_mcp(rmcp_result).context("Failed to convert MCP prompt result")
    }

    pub async fn shutdown(&self) -> Result<()> {
        let mut guard = self.state.lock().await;
        let state = std::mem::replace(&mut *guard, ClientState::Stopped);
        drop(guard);

        match state {
            ClientState::Ready { service } => {
                service.cancellation_token().cancel();
                Ok(())
            }
            ClientState::Connecting { mut transport } => {
                drop(transport.take());
                Ok(())
            }
            ClientState::Stopped => Ok(()),
        }
    }

    async fn service(&self) -> Result<Arc<RunningService<RoleClient, LoggingClientHandler>>> {
        let guard = self.state.lock().await;
        match &*guard {
            ClientState::Ready { service } => Ok(service.clone()),
            ClientState::Connecting { .. } => Err(anyhow!("MCP client not initialized")),
            ClientState::Stopped => Err(anyhow!("MCP client has been shut down")),
        }
    }
}

async fn run_with_timeout<F, T>(fut: F, timeout: Option<Duration>, label: &str) -> Result<T>
where
    F: std::future::Future<Output = Result<T, rmcp::service::ServiceError>>,
{
    if let Some(duration) = timeout {
        let result = time::timeout(duration, fut)
            .await
            .with_context(|| anyhow!("Timed out awaiting {label} after {duration:?}"))?;
        result.map_err(|err| anyhow!("{label} failed: {err}"))
    } else {
        fut.await.map_err(|err| anyhow!("{label} failed: {err}"))
    }
}

fn convert_call_tool_result(result: rmcp::model::CallToolResult) -> Result<CallToolResult> {
    let mut value = serde_json::to_value(result)?;
    if let Some(obj) = value.as_object_mut() {
        let missing_or_null = obj.get("content").map(Value::is_null).unwrap_or(true);
        if missing_or_null {
            obj.insert("content".to_string(), Value::Array(Vec::new()));
        }
    }
    serde_json::from_value(value).context("Failed to convert call tool result")
}

pub(crate) fn convert_to_rmcp<T, U>(value: T) -> Result<U>
where
    T: serde::Serialize,
    U: serde::de::DeserializeOwned,
{
    let json = serde_json::to_value(value)?;
    serde_json::from_value(json).map_err(|err| anyhow!(err))
}

pub(crate) fn convert_to_mcp<T, U>(value: T) -> Result<U>
where
    T: serde::Serialize,
    U: serde::de::DeserializeOwned,
{
    let json = serde_json::to_value(value)?;
    serde_json::from_value(json).map_err(|err| anyhow!(err))
}

fn create_env_for_mcp_server(
    extra_env: Option<HashMap<String, String>>,
) -> HashMap<String, String> {
    DEFAULT_ENV_VARS
        .iter()
        .filter_map(|var| {
            std::env::var(var)
                .ok()
                .map(|value| (var.to_string(), value))
        })
        .chain(extra_env.unwrap_or_default())
        .collect()
}

#[cfg(unix)]
const DEFAULT_ENV_VARS: &[&str] = &[
    "HOME",
    "LOGNAME",
    "PATH",
    "SHELL",
    "USER",
    "__CF_USER_TEXT_ENCODING",
    "LANG",
    "LC_ALL",
    "TERM",
    "TMPDIR",
    "TZ",
];

#[cfg(windows)]
const DEFAULT_ENV_VARS: &[&str] = &[
    // Core path resolution
    "PATH",
    "PATHEXT",
    // Shell and system roots
    "COMSPEC",
    "SYSTEMROOT",
    "SYSTEMDRIVE",
    // User context and profiles
    "USERNAME",
    "USERDOMAIN",
    "USERPROFILE",
    "HOMEDRIVE",
    "HOMEPATH",
    // Program locations
    "PROGRAMFILES",
    "PROGRAMFILES(X86)",
    "PROGRAMW6432",
    "PROGRAMDATA",
    // App data and caches
    "LOCALAPPDATA",
    "APPDATA",
    // Temp locations
    "TEMP",
    "TMP",
    // Common shells/pwsh hints
    "POWERSHELL",
    "PWSH",
];
