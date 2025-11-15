//! Test infrastructure for bash runner

#[cfg(test)]
mod tests {
    use super::super::BashRunner;
    use crate::executor::{CommandExecutor, CommandInvocation, CommandOutput, CommandStatus, CommandCategory};
    use crate::policy::AllowAllPolicy;
    use anyhow::Result;
    use assert_fs::TempDir;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    pub struct RecordingExecutor {
        invocations: Arc<Mutex<Vec<CommandInvocation>>>,
    }

    impl CommandExecutor for RecordingExecutor {
        fn execute(&self, invocation: &CommandInvocation) -> Result<CommandOutput> {
            self.invocations.lock().unwrap().push(invocation.clone());
            Ok(CommandOutput {
                status: CommandStatus::new(true, Some(0)),
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    #[test]
    fn cd_updates_working_directory() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        let runner = BashRunner::new(
            dir.path().to_path_buf(),
            RecordingExecutor::default(),
            AllowAllPolicy,
        );
        let mut runner = runner.unwrap();
        runner.cd("nested").unwrap();
        assert_eq!(runner.working_dir(), nested);
    }

    #[test]
    fn mkdir_records_invocation() {
        let dir = TempDir::new().unwrap();
        let executor = RecordingExecutor::default();
        let runner = BashRunner::new(dir.path().to_path_buf(), executor.clone(), AllowAllPolicy);
        runner.unwrap().mkdir("new_dir", true).unwrap();
        let invocations = executor.invocations.lock().unwrap();
        assert_eq!(invocations.len(), 1);
        assert_eq!(invocations[0].category, CommandCategory::CreateDirectory);
    }
}
