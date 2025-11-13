use std::error::Error;

use vtcode_exec_events::{
    CommandExecutionItem, CommandExecutionStatus, ItemCompletedEvent, ItemStartedEvent,
    ItemUpdatedEvent, ThreadEvent, ThreadItem, ThreadItemDetails, ThreadStartedEvent,
    TurnCompletedEvent, TurnStartedEvent, Usage,
};

fn main() -> Result<(), Box<dyn Error>> {
    let timeline = sample_timeline();

    println!("# execution timeline (JSONL)");
    for event in &timeline {
        let json = serde_json::to_string(event)?;
        println!("{}", json);
    }

    println!("\n# completed commands");
    for event in &timeline {
        if let ThreadEvent::ItemCompleted(ItemCompletedEvent { item }) = event {
            if let ThreadItemDetails::CommandExecution(command) = &item.details {
                println!(
                    "{} => status={} exit_code={:?}",
                    command.command,
                    status_label(&command.status),
                    command.exit_code
                );
            }
        }
    }

    Ok(())
}

fn sample_timeline() -> Vec<ThreadEvent> {
    /// Helper to create a command execution ThreadItem with the given parameters.
    fn make_command_item(
        id: &str,
        command: &str,
        output: &str,
        exit_code: Option<i32>,
        status: CommandExecutionStatus,
    ) -> ThreadItem {
        ThreadItem {
            id: id.into(),
            details: ThreadItemDetails::CommandExecution(CommandExecutionItem {
                command: command.into(),
                aggregated_output: output.into(),
                exit_code,
                status,
            }),
        }
    }

    let command_id = "command.git-init";
    let command_str = "git init";
    let final_output = "Initialized empty Git repository";

    vec![
        ThreadEvent::ThreadStarted(ThreadStartedEvent {
            thread_id: "workspace.setup".into(),
        }),
        ThreadEvent::TurnStarted(TurnStartedEvent::default()),
        ThreadEvent::ItemStarted(ItemStartedEvent {
            item: make_command_item(
                command_id,
                command_str,
                "",
                None,
                CommandExecutionStatus::InProgress,
            ),
        }),
        ThreadEvent::ItemUpdated(ItemUpdatedEvent {
            item: make_command_item(
                command_id,
                command_str,
                final_output,
                None,
                CommandExecutionStatus::InProgress,
            ),
        }),
        ThreadEvent::ItemCompleted(ItemCompletedEvent {
            item: make_command_item(
                command_id,
                command_str,
                final_output,
                Some(0),
                CommandExecutionStatus::Completed,
            ),
        }),
        ThreadEvent::TurnCompleted(TurnCompletedEvent {
            usage: Usage {
                input_tokens: 128,
                cached_input_tokens: 0,
                output_tokens: 32,
            },
        }),
    ]
}

fn status_label(status: &CommandExecutionStatus) -> &'static str {
    match status {
        CommandExecutionStatus::Completed => "completed",
        CommandExecutionStatus::Failed => "failed",
        CommandExecutionStatus::InProgress => "in_progress",
    }
}
