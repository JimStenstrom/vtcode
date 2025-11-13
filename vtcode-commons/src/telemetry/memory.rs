use anyhow::Result;

use crate::buffer::MemoryBuffer;
use super::TelemetrySink;

/// In-memory telemetry sink that records cloned events for later inspection.
///
/// This helper is primarily intended for tests, examples, or prototypes that
/// want to assert on the events emitted by a component without integrating a
/// full telemetry backend. The recorded events can be retrieved via
/// [`MemoryTelemetry::take`].
#[derive(Debug, Default, Clone)]
pub struct MemoryTelemetry<Event> {
    buffer: MemoryBuffer<Event>,
}

impl<Event> MemoryTelemetry<Event> {
    /// Creates a new memory-backed telemetry sink.
    pub fn new() -> Self {
        Self {
            buffer: MemoryBuffer::new(),
        }
    }

    /// Returns the recorded events, draining the internal buffer.
    pub fn take(&self) -> Vec<Event> {
        self.buffer.take()
    }
}

impl<Event> TelemetrySink<Event> for MemoryTelemetry<Event>
where
    Event: Clone + Send + Sync,
{
    fn record(&self, event: &Event) -> Result<()> {
        self.buffer.push(event.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_telemetry_records_events() {
        let telemetry = MemoryTelemetry::new();
        telemetry.record(&"event-1").unwrap();
        telemetry.record(&"event-2").unwrap();

        assert_eq!(telemetry.take(), vec!["event-1", "event-2"]);
        assert!(telemetry.take().is_empty());
    }
}
