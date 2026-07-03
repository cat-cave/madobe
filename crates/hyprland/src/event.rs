use crate::{CommandContext, HyprlandError, HyprlandErrorKind, Result};
use std::io::BufRead;

/// Parsed Hyprland socket2 event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HyprlandEvent {
    name: String,
    payload: String,
}

impl HyprlandEvent {
    /// Creates an event value.
    #[must_use]
    pub fn new(name: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            payload: payload.into(),
        }
    }

    /// Returns the event name before `>>`.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the event payload after `>>`.
    #[must_use]
    pub fn payload(&self) -> &str {
        &self.payload
    }
}

/// Parses one Hyprland socket2 event line.
///
/// # Errors
///
/// Returns [`HyprlandErrorKind::InvalidEvent`] when the line does not contain
/// an event separator or has an empty event name.
pub fn parse_event_line(line: &str) -> Result<HyprlandEvent> {
    let line = line.trim_end_matches(['\r', '\n']);
    let Some((name, payload)) = line.split_once(">>") else {
        return Err(invalid_event("event line must contain '>>'"));
    };

    if name.is_empty() {
        return Err(invalid_event("event name must not be empty"));
    }

    Ok(HyprlandEvent::new(name, payload))
}

/// Pull-based source of Hyprland events.
pub trait EventSource {
    /// Returns the next event, or `None` on end of stream.
    ///
    /// # Errors
    ///
    /// Returns [`HyprlandError`] when reading or parsing the event stream fails.
    fn next_event(&mut self) -> Result<Option<HyprlandEvent>>;
}

/// Event source over newline-delimited socket2 event text.
#[derive(Debug)]
pub struct LinesEventSource<R> {
    reader: R,
}

impl<R> LinesEventSource<R>
where
    R: BufRead,
{
    /// Creates a line event source.
    #[must_use]
    pub const fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R> EventSource for LinesEventSource<R>
where
    R: BufRead,
{
    fn next_event(&mut self) -> Result<Option<HyprlandEvent>> {
        let mut line = String::new();
        let bytes = self.reader.read_line(&mut line).map_err(|error| {
            HyprlandError::new(
                CommandContext::new("hyprland-socket2", ["events"]),
                HyprlandErrorKind::EventIo {
                    message: error.to_string(),
                },
            )
        })?;

        if bytes == 0 {
            return Ok(None);
        }

        parse_event_line(&line).map(Some)
    }
}

fn invalid_event(reason: &str) -> HyprlandError {
    HyprlandError::new(
        CommandContext::new("hyprland-socket2", ["events"]),
        HyprlandErrorKind::InvalidEvent {
            reason: reason.to_owned(),
        },
    )
}
