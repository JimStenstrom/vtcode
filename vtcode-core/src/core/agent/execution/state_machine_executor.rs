//! State machine execution orchestrator
//!
//! This module provides the main state machine execution loop that coordinates
//! all the state handlers to execute agent tasks.

use super::{AgentState, ExecutionContext, Transition};
use crate::config::constants::tools;
use crate::config::models::{ModelId, Provider as ModelProvider};
use crate::core::agent::events::ExecEventRecorder;
use crate::core::agent::runner::format_tool_result_for_display;
use crate::core::agent::task::TaskOutcome;
use crate::exec::events::CommandExecutionStatus;
use crate::llm::provider::{Message, ToolDefinition};
use crate::utils::colors::style;
use anyhow::{Result, anyhow};
use serde_json::Value;
use vtcode_llm_gemini::{Content, Part};

/// Orchestrates state machine execution
pub struct StateMachineExecutor;

impl StateMachineExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute the state machine loop
    pub async fn run(
        &self,
        mut state: AgentState,
        ctx: &mut ExecutionContext,
        system_instruction: &str,
        conversation: &mut Vec<Content>,
        conversation_messages: &mut Vec<Message>,
        tools: &[ToolDefinition],
        provider: &dyn crate::llm::provider::LLMProvider,
        tool_registry: &crate::tools::ToolRegistry,
        agent_type: crate::core::agent::types::AgentType,
        model: &str,
        reasoning_effort: Option<crate::config::types::ReasoningEffortLevel>,
        event_recorder: &mut ExecEventRecorder,
        quiet: bool,
    ) -> Result<TaskOutcome> {
        let message_builder = super::MessageBuilder::new();
        let llm_caller = super::LLMCaller::new();
        let tool_executor = super::ToolExecutor::new();
        let loop_controller = super::LoopController::new();
        let response_processor = super::ResponseProcessor::new();

        let agent_prefix = format!("[{}]", agent_type);
        let provider_kind = model
            .parse::<ModelId>()
            .map(|m| m.provider())
            .unwrap_or(ModelProvider::Gemini);

        let parallel_tool_config = if provider.supports_parallel_tool_config(model) {
            Some(crate::llm::provider::ParallelToolConfig::anthropic_optimized())
        } else {
            None
        };

        // Main state machine loop
        loop {
            // Log state transition for debugging
            tracing::debug!("State transition: {}", state.description());

            // Execute current state and get transition
            let transition: Transition = match state {
                AgentState::PreparingMessages => {
                    message_builder.prepare_messages(system_instruction, conversation, ctx)?
                }

                AgentState::CallingLLM {
                    ref messages,
                    ref system_instruction,
                    ..
                } => {
                    if !quiet {
                        println!(
                            "{} {} calling LLM...",
                            agent_prefix,
                            style("(LLM)").cyan().bold()
                        );
                    }

                    llm_caller
                        .call_llm(
                            provider,
                            messages.clone(),
                            system_instruction.clone(),
                            tools,
                            model,
                            reasoning_effort,
                            parallel_tool_config.clone(),
                            ctx.supports_streaming,
                            ctx,
                        )
                        .await?
                }

                AgentState::ProcessingResponse { ref response } => {
                    let processed = response_processor.process_response(
                        response.clone(),
                        provider_kind,
                        ctx.turn_number,
                        ctx,
                    )?;

                    // Record agent message if present
                    if !processed.response_text.trim().is_empty() {
                        if !quiet {
                            println!(
                                "{} [{}]: {}",
                                style("[RESPONSE]").cyan().bold(),
                                agent_type,
                                processed.response_text.trim()
                            );
                        }
                        event_recorder.agent_message(&processed.response_text);
                    }

                    // Record reasoning if present
                    if let Some(ref reasoning) = processed.reasoning {
                        event_recorder.reasoning(reasoning);
                    }

                    // Add to conversation
                    if !processed.response_text.trim().is_empty() {
                        conversation.push(Content {
                            role: "model".to_string(),
                            parts: vec![Part::Text {
                                text: processed.response_text.clone(),
                            }],
                        });
                    }

                    // Check for completion detection in response
                    if loop_controller.detect_completion(&processed.response_text) {
                        ctx.task_completed = true;
                    }

                    response_processor.next_state(&processed)
                }

                AgentState::ExecutingTool {
                    ref tool_name,
                    ref tool_args,
                    ref call_id,
                } => {
                    if !quiet {
                        println!(
                            "{} [{}] Calling tool: {}",
                            style("[TOOL_CALL]").blue().bold(),
                            agent_type,
                            tool_name
                        );
                    }

                    tool_executor
                        .execute_tool(
                            tool_registry,
                            agent_type,
                            tool_name.clone(),
                            tool_args.clone(),
                            call_id.clone(),
                            ctx,
                        )
                        .await?
                }

                AgentState::RecordingToolResult {
                    ref tool_name,
                    ref call_id,
                    ref result,
                } => {
                    let command_event = event_recorder.command_started(tool_name);

                    match result {
                        super::ToolResult::Success(result_value) => {
                            if !quiet {
                                println!(
                                    "{} {}",
                                    agent_prefix,
                                    format!(
                                        "{} {} tool executed successfully",
                                        style("(OK)").green(),
                                        tool_name
                                    )
                                );
                            }

                            let tool_result = serde_json::to_string(result_value)?;
                            let display_text = format_tool_result_for_display(tool_name, result_value);

                            // Add to conversation
                            conversation.push(Content {
                                role: "user".to_string(),
                                parts: vec![Part::Text { text: display_text }],
                            });

                            conversation_messages.push(Message::tool_response(
                                call_id.clone(),
                                tool_result.clone(),
                            ));

                            event_recorder.command_finished(
                                &command_event,
                                CommandExecutionStatus::Completed,
                                None,
                                "",
                            );

                            // Track file modifications
                            if tool_name == tools::WRITE_FILE {
                                if let Some(filepath) = result_value.get("path").and_then(|p| p.as_str())
                                {
                                    event_recorder.file_change_completed(filepath);
                                }
                            }

                            // Register tool loop
                            let loops = ctx.register_tool_loop();
                            if loops >= ctx.max_tool_loops {
                                let warning_message = format!(
                                    "Reached tool-call limit of {} iterations; pausing autonomous loop",
                                    ctx.max_tool_loops
                                );
                                if !quiet {
                                    println!(
                                        "{} {}",
                                        agent_prefix,
                                        format!("{} {}", style("(WARN)").yellow().bold(), warning_message)
                                    );
                                }
                                event_recorder.warning(&warning_message);
                                Transition::Terminal(TaskOutcome::ToolLoopLimitReached)
                            } else {
                                Transition::Continue(AgentState::CheckingContinuation)
                            }
                        }
                        super::ToolResult::Error(error_msg) => {
                            if !quiet {
                                println!(
                                    "{} {}",
                                    agent_prefix,
                                    format!(
                                        "{} {} tool failed: {}",
                                        style("(ERR)").red(),
                                        tool_name,
                                        error_msg
                                    )
                                );
                            }

                            let warning_message = format!("Tool {} failed: {}", tool_name, error_msg);
                            event_recorder.command_finished(
                                &command_event,
                                CommandExecutionStatus::Failed,
                                None,
                                &warning_message,
                            );
                            event_recorder.warning(&warning_message);

                            // Add error to conversation
                            conversation.push(Content {
                                role: "user".to_string(),
                                parts: vec![Part::Text {
                                    text: format!("Tool {} failed: {}", tool_name, error_msg),
                                }],
                            });

                            let error_payload = serde_json::json!({ "error": error_msg }).to_string();
                            conversation_messages.push(Message::tool_response(
                                call_id.clone(),
                                error_payload,
                            ));

                            Transition::Continue(AgentState::CheckingContinuation)
                        }
                    }
                }

                AgentState::CheckingContinuation => {
                    // Determine if we had tool calls in the last response
                    let had_tool_call = ctx.consecutive_tool_loops > 0;

                    // Get last response text from conversation
                    let response_text = conversation
                        .iter()
                        .rev()
                        .find(|c| c.role == "model")
                        .and_then(|c| {
                            c.parts.iter().find_map(|p| match p {
                                Part::Text { text } => Some(text.as_str()),
                                _ => None,
                            })
                        })
                        .unwrap_or("");

                    loop_controller.check_continuation(had_tool_call, response_text, ctx)?
                }

                AgentState::Completed { ref outcome } => {
                    return Ok(outcome.clone());
                }

                AgentState::Failed { ref error } => {
                    return Err(anyhow!(error.clone()));
                }
            };

            // Transition to next state
            state = match transition {
                Transition::Continue(next_state) => next_state,
                Transition::Terminal(outcome) => {
                    return Ok(outcome);
                }
            };

            // Safety check - detect infinite loops
            if ctx.turn_number > ctx.max_turns * 2 {
                return Err(anyhow!("Safety limit exceeded - possible infinite loop detected"));
            }
        }
    }
}

impl Default for StateMachineExecutor {
    fn default() -> Self {
        Self::new()
    }
}
