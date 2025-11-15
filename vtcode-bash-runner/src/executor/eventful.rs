//! Event-emitting executor wrapper

use anyhow::Result;
use std::sync::{
    Mutex as StdMutex,
    atomic::{AtomicU64, Ordering},
};

use super::{CommandExecutor, CommandInvocation, CommandOutput};
use vtcode_exec_events::{
    CommandExecutionItem, CommandExecutionStatus, EventEmitter, ItemCompletedEvent,
    ItemStartedEvent, ThreadEvent, ThreadItem, ThreadItemDetails,
};

/// Executor wrapper that emits events for command execution.
///
/// Wraps any `CommandExecutor` and emits start/completion events via an `EventEmitter`.
/// Useful for tracking command execution progress in UI or logging systems.
#[derive(Debug)]
pub struct EventfulExecutor<E, T> {
    inner: E,
    emitter: StdMutex<T>,
    counter: AtomicU64,
    id_prefix: String,
}

impl<E, T> EventfulExecutor<E, T>
where
    T: EventEmitter,
{
    pub fn new(inner: E, emitter: T) -> Self {
        Self {
            inner,
            emitter: StdMutex::new(emitter),
            counter: AtomicU64::new(0),
            id_prefix: "cmd-".to_string(),
        }
    }

    pub fn with_id_prefix(inner: E, emitter: T, prefix: impl Into<String>) -> Self {
        let mut executor = Self::new(inner, emitter);
        executor.id_prefix = prefix.into();
        executor
    }

    fn next_id(&self) -> String {
        let value = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        format!("{}{}", self.id_prefix, value)
    }

    fn emit_event(&self, event: ThreadEvent) {
        if let Ok(mut emitter) = self.emitter.lock() {
            EventEmitter::emit(&mut *emitter, &event);
        }
    }

    fn command_details(
        &self,
        invocation: &CommandInvocation,
        status: CommandExecutionStatus,
        output: Option<&CommandOutput>,
        error: Option<&anyhow::Error>,
    ) -> CommandExecutionItem {
        let aggregated_output = if let Some(output) = output {
            super::helpers::aggregate_output(output)
        } else if let Some(err) = error {
            err.to_string()
        } else {
            String::new()
        };

        CommandExecutionItem {
            command: invocation.command.clone(),
            aggregated_output,
            exit_code: output.and_then(|out| out.status.code()),
            status,
        }
    }
}

impl<E, T> CommandExecutor for EventfulExecutor<E, T>
where
    E: CommandExecutor,
    T: EventEmitter + Send,
{
    fn execute(&self, invocation: &CommandInvocation) -> Result<CommandOutput> {
        let item_id = self.next_id();
        let starting_item = ThreadItem {
            id: item_id.clone(),
            details: ThreadItemDetails::CommandExecution(self.command_details(
                invocation,
                CommandExecutionStatus::InProgress,
                None,
                None,
            )),
        };
        self.emit_event(ThreadEvent::ItemStarted(ItemStartedEvent {
            item: starting_item,
        }));

        match self.inner.execute(invocation) {
            Ok(output) => {
                let status = if output.status.success() {
                    CommandExecutionStatus::Completed
                } else {
                    CommandExecutionStatus::Failed
                };

                let completed_item = ThreadItem {
                    id: item_id,
                    details: ThreadItemDetails::CommandExecution(self.command_details(
                        invocation,
                        status,
                        Some(&output),
                        None,
                    )),
                };
                self.emit_event(ThreadEvent::ItemCompleted(ItemCompletedEvent {
                    item: completed_item,
                }));
                Ok(output)
            }
            Err(err) => {
                let failure = ThreadItem {
                    id: item_id,
                    details: ThreadItemDetails::CommandExecution(self.command_details(
                        invocation,
                        CommandExecutionStatus::Failed,
                        None,
                        Some(&err),
                    )),
                };
                self.emit_event(ThreadEvent::ItemCompleted(ItemCompletedEvent {
                    item: failure,
                }));
                Err(err)
            }
        }
    }
}
