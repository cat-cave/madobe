#![doc = "Mockable Hyprland command and event layer for madobe."]
#![forbid(unsafe_code)]
#![allow(
    clippy::module_name_repetitions,
    reason = "The public API names Hyprland types explicitly to keep backend boundaries clear."
)]

mod client;
mod command;
mod error;
mod event;
mod model;

pub use client::HyprlandClient;
pub use command::{
    CommandExecutor, CommandOutput, HyprctlCommand, HyprctlEndpoint, HyprctlExecutor,
};
pub use error::{CommandContext, HyprlandError, HyprlandErrorKind, Result};
pub use event::{EventSource, HyprlandEvent, LinesEventSource, parse_event_line};
pub use model::{
    HyprlandMonitor, HyprlandWorkspace, HyprlandWorkspaceRef, MonitorPosition,
    parse_active_workspace, parse_active_workspace_with_context, parse_monitors,
    parse_monitors_with_context, parse_workspaces, parse_workspaces_with_context,
};
