use anyhow::{Error, Result};

use crate::buffer::MemoryBuffer;
use super::ErrorReporter;

/// Simple [`ErrorReporter`] that stores error messages in memory.
///
/// This helper is designed for tests and examples that need to assert on the
/// errors emitted by a component without wiring an external monitoring system.
/// Callers can retrieve captured messages via [`MemoryErrorReporter::take`].
#[derive(Debug, Default, Clone)]
pub struct MemoryErrorReporter {
    buffer: MemoryBuffer<String>,
}

impl MemoryErrorReporter {
    /// Creates a new memory-backed error reporter.
    pub fn new() -> Self {
        Self {
            buffer: MemoryBuffer::new(),
        }
    }

    /// Returns the captured error messages, draining the buffer.
    pub fn take(&self) -> Vec<String> {
        self.buffer.take()
    }
}

impl ErrorReporter for MemoryErrorReporter {
    fn capture(&self, error: &Error) -> Result<()> {
        self.buffer.push(format!("{error:?}"));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_error_reporter_captures_messages() {
        let reporter = MemoryErrorReporter::new();
        reporter.capture(&Error::msg("error-1")).unwrap();
        reporter.capture(&Error::msg("error-2")).unwrap();

        let messages = reporter.take();
        assert_eq!(messages.len(), 2);
        assert!(messages[0].contains("error-1"));
        assert!(messages[1].contains("error-2"));
        assert!(reporter.take().is_empty());
    }
}
