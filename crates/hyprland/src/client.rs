use crate::model::{
    parse_active_workspace_with_context, parse_monitors_with_context, parse_workspaces_with_context,
};
use crate::{
    CommandExecutor, HyprctlCommand, HyprctlEndpoint, HyprlandMonitor, HyprlandWorkspace, Result,
};

/// Hyprland command client backed by an injectable executor.
#[derive(Clone, Debug)]
pub struct HyprlandClient<E> {
    executor: E,
}

impl<E> HyprlandClient<E> {
    /// Creates a client from a command executor.
    #[must_use]
    pub const fn new(executor: E) -> Self {
        Self { executor }
    }

    /// Returns the underlying executor.
    #[must_use]
    pub const fn executor(&self) -> &E {
        &self.executor
    }

    /// Consumes the client and returns the underlying executor.
    #[must_use]
    pub fn into_executor(self) -> E {
        self.executor
    }
}

impl<E> HyprlandClient<E>
where
    E: CommandExecutor,
{
    /// Runs an arbitrary `hyprctl` command through the executor.
    ///
    /// # Errors
    ///
    /// Returns [`crate::HyprlandError`] when command execution fails.
    pub fn run(&self, command: &HyprctlCommand) -> Result<String> {
        Ok(self.executor.execute(command)?.stdout().to_owned())
    }

    /// Lists Hyprland monitors.
    ///
    /// # Errors
    ///
    /// Returns [`crate::HyprlandError`] when the command or JSON parsing fails.
    pub fn monitors(&self) -> Result<Vec<HyprlandMonitor>> {
        let command = HyprctlCommand::json(HyprctlEndpoint::Monitors);
        let output = self.executor.execute(&command)?;
        parse_monitors_with_context(output.stdout(), command.context())
    }

    /// Lists Hyprland workspaces.
    ///
    /// # Errors
    ///
    /// Returns [`crate::HyprlandError`] when the command or JSON parsing fails.
    pub fn workspaces(&self) -> Result<Vec<HyprlandWorkspace>> {
        let command = HyprctlCommand::json(HyprctlEndpoint::Workspaces);
        let output = self.executor.execute(&command)?;
        parse_workspaces_with_context(output.stdout(), command.context())
    }

    /// Returns Hyprland's active workspace.
    ///
    /// # Errors
    ///
    /// Returns [`crate::HyprlandError`] when the command or JSON parsing fails.
    pub fn active_workspace(&self) -> Result<HyprlandWorkspace> {
        let command = HyprctlCommand::json(HyprctlEndpoint::ActiveWorkspace);
        let output = self.executor.execute(&command)?;
        parse_active_workspace_with_context(output.stdout(), command.context())
    }
}
