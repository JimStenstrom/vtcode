//! Helper functions for command execution

/// Aggregates stdout and stderr into a single string, trimming whitespace.
#[cfg(feature = "exec-events")]
pub(crate) fn aggregate_output(output: &super::types::CommandOutput) -> String {
    let mut combined = String::new();
    if !output.stdout.trim().is_empty() {
        combined.push_str(output.stdout.trim());
    }
    if !output.stderr.trim().is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(output.stderr.trim());
    }
    combined
}
