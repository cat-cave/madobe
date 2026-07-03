use crate::{CommandContext, HyprlandError, HyprlandErrorKind, Result};
use std::process::Command;

const HYPRCTL: &str = "hyprctl";

/// `hyprctl -j` endpoints parsed by this crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HyprctlEndpoint {
    /// `hyprctl -j monitors`.
    Monitors,
    /// `hyprctl -j workspaces`.
    Workspaces,
    /// `hyprctl -j activeworkspace`.
    ActiveWorkspace,
}

impl HyprctlEndpoint {
    /// Returns the endpoint argument passed to `hyprctl`.
    #[must_use]
    pub const fn as_arg(self) -> &'static str {
        match self {
            Self::Monitors => "monitors",
            Self::Workspaces => "workspaces",
            Self::ActiveWorkspace => "activeworkspace",
        }
    }
}

/// Command invocation accepted by a [`CommandExecutor`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HyprctlCommand {
    args: Vec<String>,
}

impl HyprctlCommand {
    /// Creates a JSON `hyprctl` command for an endpoint.
    #[must_use]
    pub fn json(endpoint: HyprctlEndpoint) -> Self {
        Self {
            args: vec!["-j".to_owned(), endpoint.as_arg().to_owned()],
        }
    }

    /// Creates an arbitrary `hyprctl` command.
    ///
    /// # Errors
    ///
    /// Returns [`HyprlandErrorKind::InvalidCommand`] when any argument is empty.
    pub fn new(args: impl IntoIterator<Item = impl Into<String>>) -> Result<Self> {
        let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
        if args.iter().any(String::is_empty) {
            return Err(HyprlandError::new(
                CommandContext::new(HYPRCTL, args),
                HyprlandErrorKind::InvalidCommand {
                    reason: "hyprctl arguments must not be empty".to_owned(),
                },
            ));
        }

        Ok(Self { args })
    }

    /// Returns the executable context for diagnostics.
    #[must_use]
    pub fn context(&self) -> CommandContext {
        CommandContext::new(HYPRCTL, self.args.clone())
    }

    /// Returns command arguments.
    #[must_use]
    pub fn args(&self) -> &[String] {
        &self.args
    }
}

/// Captured output from a successful command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutput {
    stdout: String,
    stderr: String,
}

impl CommandOutput {
    /// Creates command output.
    #[must_use]
    pub fn new(stdout: impl Into<String>, stderr: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }

    /// Returns captured standard output.
    #[must_use]
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Returns captured standard error.
    #[must_use]
    pub fn stderr(&self) -> &str {
        &self.stderr
    }
}

/// Injectable command execution boundary for Hyprland commands.
pub trait CommandExecutor {
    /// Executes a Hyprland command.
    ///
    /// # Errors
    ///
    /// Returns [`HyprlandError`] when command execution fails.
    fn execute(&self, command: &HyprctlCommand) -> Result<CommandOutput>;
}

/// Real `hyprctl` executor.
#[derive(Clone, Copy, Debug, Default)]
pub struct HyprctlExecutor;

impl CommandExecutor for HyprctlExecutor {
    fn execute(&self, command: &HyprctlCommand) -> Result<CommandOutput> {
        let context = command.context();
        let output = Command::new(HYPRCTL)
            .args(command.args())
            .output()
            .map_err(|error| {
                HyprlandError::new(
                    context.clone(),
                    HyprlandErrorKind::Spawn {
                        message: error.to_string(),
                    },
                )
            })?;

        let stderr = String::from_utf8(output.stderr).map_err(|error| {
            HyprlandError::new(
                context.clone(),
                HyprlandErrorKind::InvalidUtf8 {
                    message: error.to_string(),
                },
            )
        })?;

        if !output.status.success() {
            return Err(HyprlandError::new(
                context,
                HyprlandErrorKind::NonZeroExit {
                    code: output.status.code(),
                    stderr,
                },
            ));
        }

        let stdout = String::from_utf8(output.stdout).map_err(|error| {
            HyprlandError::new(
                context,
                HyprlandErrorKind::InvalidUtf8 {
                    message: error.to_string(),
                },
            )
        })?;

        Ok(CommandOutput::new(stdout, stderr))
    }
}
