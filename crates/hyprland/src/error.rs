use std::error::Error;
use std::fmt::{self, Display};

/// Result alias for Hyprland command and parse operations.
pub type Result<T> = std::result::Result<T, HyprlandError>;

/// Command context preserved on Hyprland failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandContext {
    program: String,
    args: Vec<String>,
}

impl CommandContext {
    /// Creates command context from a program and arguments.
    #[must_use]
    pub fn new(
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns context for parser-only entry points.
    #[must_use]
    pub fn parser(endpoint: &str) -> Self {
        Self::new("hyprctl", ["-j", endpoint])
    }

    /// Returns the executable name.
    #[must_use]
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Returns the executable arguments.
    #[must_use]
    pub fn args(&self) -> &[String] {
        &self.args
    }
}

/// Typed Hyprland command/event failure reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HyprlandErrorKind {
    /// The command could not be spawned.
    Spawn {
        /// Operating-system error message.
        message: String,
    },
    /// The command exited unsuccessfully.
    NonZeroExit {
        /// Exit code when the process exited normally.
        code: Option<i32>,
        /// Captured standard error.
        stderr: String,
    },
    /// The command output was not valid UTF-8.
    InvalidUtf8 {
        /// UTF-8 decoding error message.
        message: String,
    },
    /// The command returned malformed JSON for the requested shape.
    InvalidJson {
        /// JSON decoding error message.
        message: String,
    },
    /// The requested command could not be represented safely.
    InvalidCommand {
        /// Command validation failure reason.
        reason: String,
    },
    /// A Hyprland socket event line was malformed.
    InvalidEvent {
        /// Event parsing failure reason.
        reason: String,
    },
    /// Reading an event stream failed.
    EventIo {
        /// I/O error message.
        message: String,
    },
}

/// Hyprland failure with command context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HyprlandError {
    context: CommandContext,
    kind: HyprlandErrorKind,
}

impl HyprlandError {
    /// Creates a Hyprland error.
    #[must_use]
    pub const fn new(context: CommandContext, kind: HyprlandErrorKind) -> Self {
        Self { context, kind }
    }

    /// Returns the command context.
    #[must_use]
    pub const fn context(&self) -> &CommandContext {
        &self.context
    }

    /// Returns the typed failure reason.
    #[must_use]
    pub const fn kind(&self) -> &HyprlandErrorKind {
        &self.kind
    }
}

impl Display for HyprlandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {:?} failed: {:?}",
            self.context.program, self.context.args, self.kind
        )
    }
}

impl Error for HyprlandError {}
