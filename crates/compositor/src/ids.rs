use std::error::Error;
use std::fmt::{self, Display};

/// Stable identifier for a compositor-managed remote output.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OutputId(String);

impl OutputId {
    /// Creates an output identifier.
    ///
    /// # Errors
    ///
    /// Returns [`IdentifierError`] when the identifier is empty or contains
    /// leading or trailing whitespace.
    pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        validate_identifier(&value)?;
        Ok(Self(value))
    }

    /// Returns the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for OutputId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stable identifier for an application/session workspace.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkspaceId(String);

impl WorkspaceId {
    /// Creates a workspace identifier.
    ///
    /// # Errors
    ///
    /// Returns [`IdentifierError`] when the identifier is empty or contains
    /// leading or trailing whitespace.
    pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        validate_identifier(&value)?;
        Ok(Self(value))
    }

    /// Returns the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for WorkspaceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stable identifier for a remote client session.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SessionId(String);

impl SessionId {
    /// Creates a session identifier.
    ///
    /// # Errors
    ///
    /// Returns [`IdentifierError`] when the identifier is empty or contains
    /// leading or trailing whitespace.
    pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        validate_identifier(&value)?;
        Ok(Self(value))
    }

    /// Returns the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for SessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Identifier construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierError {
    /// The identifier is empty.
    Empty,
    /// The identifier includes leading or trailing whitespace.
    SurroundingWhitespace,
}

impl Display for IdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("identifier must not be empty"),
            Self::SurroundingWhitespace => {
                formatter.write_str("identifier must not contain surrounding whitespace")
            }
        }
    }
}

impl Error for IdentifierError {}

fn validate_identifier(value: &str) -> Result<(), IdentifierError> {
    if value.is_empty() {
        return Err(IdentifierError::Empty);
    }

    if value.trim() != value {
        return Err(IdentifierError::SurroundingWhitespace);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{OutputId, SessionId, WorkspaceId};

    #[test]
    fn output_id_as_str_returns_original_identifier() {
        let id = must(OutputId::new("remote-output-primary"), "valid output id");

        assert_eq!(id.as_str(), "remote-output-primary");
    }

    #[test]
    fn workspace_id_as_str_returns_original_identifier() {
        let id = must(
            WorkspaceId::new("workspace-client-42"),
            "valid workspace id",
        );

        assert_eq!(id.as_str(), "workspace-client-42");
    }

    #[test]
    fn session_id_as_str_returns_original_identifier() {
        let id = must(SessionId::new("session-alpha-7"), "valid session id");

        assert_eq!(id.as_str(), "session-alpha-7");
    }

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>, context: &str) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{context}: {error:?}"),
        }
    }
}
