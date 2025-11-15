//! MCP client handler for logging and elicitation.

use crate::client::{McpElicitationHandler, McpElicitationRequest};
use anyhow::Result;
use jsonschema::Validator;
use mcp_types::InitializeRequestParams;
use rmcp::handler::client::ClientHandler;
use rmcp::model::{
    CancelledNotificationParam, CreateElicitationRequestParam, ElicitationAction,
    ListRootsResult, LoggingLevel, LoggingMessageNotificationParam, ProgressNotificationParam,
    ResourceUpdatedNotificationParam, Root,
};
use rmcp::service::{NotificationContext, RequestContext, RoleClient};
use serde_json::{Value, json};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use url::Url;

#[derive(Clone)]
pub struct LoggingClientHandler {
    provider: String,
    initialize_params: InitializeRequestParams,
    elicitation_handler: Option<Arc<dyn McpElicitationHandler>>,
}

impl LoggingClientHandler {
    pub fn new(
        provider_name: String,
        params: InitializeRequestParams,
        elicitation_handler: Option<Arc<dyn McpElicitationHandler>>,
    ) -> Self {
        Self {
            provider: provider_name,
            initialize_params: params,
            elicitation_handler,
        }
    }

    pub fn provider_name(&self) -> &str {
        &self.provider
    }

    fn handle_logging(&self, params: LoggingMessageNotificationParam) {
        let logger = params.logger.unwrap_or_else(|| "".to_string());
        let summary = params
            .data
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| params.data.to_string());

        match params.level {
            LoggingLevel::Debug => debug!(
                provider = self.provider.as_str(),
                logger = logger.as_str(),
                summary = %summary,
                payload = ?params.data,
                "MCP provider log"
            ),
            LoggingLevel::Info | LoggingLevel::Notice => info!(
                provider = self.provider.as_str(),
                logger = logger.as_str(),
                summary = %summary,
                payload = ?params.data,
                "MCP provider log"
            ),
            LoggingLevel::Warning => warn!(
                provider = self.provider.as_str(),
                logger = logger.as_str(),
                summary = %summary,
                payload = ?params.data,
                "MCP provider warning"
            ),
            LoggingLevel::Error
            | LoggingLevel::Critical
            | LoggingLevel::Alert
            | LoggingLevel::Emergency => error!(
                provider = self.provider.as_str(),
                logger = logger.as_str(),
                summary = %summary,
                payload = ?params.data,
                "MCP provider error"
            ),
        }
    }
}

impl ClientHandler for LoggingClientHandler {
    fn create_elicitation(
        &self,
        request: CreateElicitationRequestParam,
        _context: RequestContext<RoleClient>,
    ) -> impl std::future::Future<
        Output = Result<rmcp::model::CreateElicitationResult, rmcp::ErrorData>,
    > + Send
    + '_ {
        let provider = self.provider.clone();
        let handler = self.elicitation_handler.clone();
        async move {
            let default_response = rmcp::model::CreateElicitationResult {
                action: ElicitationAction::Decline,
                content: None,
            };

            if let Some(handler) = handler {
                let schema_value = match serde_json::to_value(&request.requested_schema) {
                    Ok(value) => value,
                    Err(err) => {
                        warn!(
                            provider = provider.as_str(),
                            error = %err,
                            "Failed to serialize MCP elicitation schema; using null placeholder"
                        );
                        Value::Null
                    }
                };
                let validator = build_elicitation_validator(provider.as_str(), &schema_value);
                let message = request.message.clone();
                let payload = McpElicitationRequest {
                    message: message.clone(),
                    requested_schema: schema_value.clone(),
                };

                match handler.handle_elicitation(&provider, payload).await {
                    Ok(response) => {
                        if let Err(error) = validate_elicitation_payload(
                            provider.as_str(),
                            validator.as_ref(),
                            &response.action,
                            response.content.as_ref(),
                        ) {
                            return Err(error);
                        }
                        info!(
                            provider = provider.as_str(),
                            message = message.as_str(),
                            action = ?response.action,
                            "MCP provider elicitation handled"
                        );
                        return Ok(rmcp::model::CreateElicitationResult {
                            action: response.action,
                            content: response.content,
                        });
                    }
                    Err(err) => {
                        warn!(
                            provider = provider.as_str(),
                            message = message.as_str(),
                            error = %err,
                            "Failed to process MCP elicitation; declining"
                        );
                    }
                }
            } else {
                info!(
                    provider = provider.as_str(),
                    message = request.message.as_str(),
                    "MCP provider requested elicitation but no handler configured; declining"
                );
            }

            Ok(default_response)
        }
    }

    fn list_roots(
        &self,
        _context: RequestContext<RoleClient>,
    ) -> impl std::future::Future<Output = Result<ListRootsResult, rmcp::ErrorData>> + Send + '_
    {
        let provider = self.provider.clone();
        async move {
            let mut roots = Vec::new();
            match std::env::current_dir() {
                Ok(dir) => {
                    if let Some(uri) = directory_to_file_uri(&dir) {
                        roots.push(Root {
                            name: Some("workspace".to_string()),
                            uri,
                        });
                    } else {
                        warn!(
                            provider = provider.as_str(),
                            path = %dir.display(),
                            "Failed to convert workspace directory to file URI for MCP roots"
                        );
                    }
                }
                Err(err) => {
                    warn!(
                        provider = provider.as_str(),
                        error = %err,
                        "Failed to resolve current directory for MCP roots"
                    );
                }
            }

            Ok(ListRootsResult { roots })
        }
    }

    fn on_cancelled(
        &self,
        params: CancelledNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        debug!(
            provider = self.provider.as_str(),
            request_id = %params.request_id,
            reason = ?params.reason,
            "MCP provider cancelled request"
        );
        async move {}
    }

    fn on_progress(
        &self,
        params: ProgressNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        info!(
            provider = self.provider.as_str(),
            progress_token = ?params.progress_token,
            progress = params.progress,
            total = ?params.total,
            message = ?params.message,
            "MCP provider progress update"
        );
        async move {}
    }

    fn on_logging_message(
        &self,
        params: LoggingMessageNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        self.handle_logging(params);
        async move {}
    }

    fn on_resource_updated(
        &self,
        params: ResourceUpdatedNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        info!(
            provider = self.provider.as_str(),
            uri = params.uri.as_str(),
            "MCP resource updated"
        );
        async move {}
    }

    fn on_resource_list_changed(
        &self,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        info!(
            provider = self.provider.as_str(),
            "MCP provider reported resource list change"
        );
        async move {}
    }

    fn on_tool_list_changed(
        &self,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        info!(
            provider = self.provider.as_str(),
            "MCP provider reported tool list change"
        );
        async move {}
    }

    fn on_prompt_list_changed(
        &self,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        info!(
            provider = self.provider.as_str(),
            "MCP provider reported prompt list change"
        );
        async move {}
    }

    fn get_info(&self) -> rmcp::model::ClientInfo {
        convert_to_rmcp(self.initialize_params.clone())
            .expect("initialize params conversion should not fail")
    }
}

pub(crate) fn build_elicitation_validator(provider: &str, schema: &Value) -> Option<Validator> {
    if schema.is_null() {
        return None;
    }

    match Validator::new(schema) {
        Ok(validator) => Some(validator),
        Err(err) => {
            warn!(
                provider = provider,
                error = %err,
                "Failed to build JSON schema validator for MCP elicitation; skipping validation"
            );
            None
        }
    }
}

pub(crate) fn validate_elicitation_payload(
    provider: &str,
    validator: Option<&Validator>,
    action: &ElicitationAction,
    content: Option<&Value>,
) -> Result<(), rmcp::ErrorData> {
    if !matches!(action, ElicitationAction::Accept) {
        return Ok(());
    }

    let Some(validator) = validator else {
        return Ok(());
    };

    let Some(payload) = content else {
        warn!(
            provider = provider,
            "MCP elicitation accept action missing response content"
        );
        return Err(rmcp::ErrorData::invalid_params(
            "Elicitation response missing content for accept action",
            None,
        ));
    };

    if !validator.is_valid(payload) {
        let messages: Vec<String> = validator
            .iter_errors(payload)
            .map(|err| err.to_string())
            .collect();
        warn!(
            provider = provider,
            errors = ?messages,
            "MCP elicitation response failed schema validation"
        );
        return Err(rmcp::ErrorData::invalid_params(
            "Elicitation response failed schema validation",
            Some(json!({ "errors": messages })),
        ));
    }

    Ok(())
}

pub(crate) fn directory_to_file_uri(path: &Path) -> Option<String> {
    Url::from_directory_path(path)
        .ok()
        .map(|url| url.to_string())
}

fn convert_to_rmcp<T, U>(value: T) -> Result<U>
where
    T: serde::Serialize,
    U: serde::de::DeserializeOwned,
{
    let json = serde_json::to_value(value)?;
    serde_json::from_value(json).map_err(Into::into)
}
