#![doc = "Minimal telemetry primitives for the madobe workspace."]
#![forbid(unsafe_code)]

/// Deterministic timestamp type used by tests and bootstrap commands.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timestamp {
    unix_millis: u64,
}

impl Timestamp {
    /// Creates a timestamp from milliseconds since the Unix epoch.
    #[must_use]
    pub const fn from_unix_millis(unix_millis: u64) -> Self {
        Self { unix_millis }
    }

    /// Returns milliseconds since the Unix epoch.
    #[must_use]
    pub const fn as_unix_millis(self) -> u64 {
        self.unix_millis
    }
}

/// Minimal event record used by M0 binaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelemetryEvent {
    name: &'static str,
    timestamp: Timestamp,
}

impl TelemetryEvent {
    /// Creates a new event with a static event name.
    #[must_use]
    pub const fn new(name: &'static str, timestamp: Timestamp) -> Self {
        Self { name, timestamp }
    }

    /// Returns the event name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the event timestamp.
    #[must_use]
    pub const fn timestamp(self) -> Timestamp {
        self.timestamp
    }
}

/// Deterministic timestamp used by the M0 hello proof.
pub const BOOTSTRAP_TIMESTAMP: Timestamp = Timestamp::from_unix_millis(0);

/// Returns the canonical bootstrap event.
#[must_use]
pub const fn bootstrap_event() -> TelemetryEvent {
    TelemetryEvent::new("madobe.bootstrap", BOOTSTRAP_TIMESTAMP)
}

#[cfg(test)]
mod tests {
    use super::{BOOTSTRAP_TIMESTAMP, Timestamp, bootstrap_event};

    #[test]
    fn event_has_deterministic_timestamp() {
        let event = bootstrap_event();

        assert_eq!(event.name(), "madobe.bootstrap");
        assert_eq!(event.timestamp(), BOOTSTRAP_TIMESTAMP);
        assert_eq!(Timestamp::from_unix_millis(7).as_unix_millis(), 7);
    }
}
