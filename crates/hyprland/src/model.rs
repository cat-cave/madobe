use crate::{CommandContext, Result};

mod parser;

/// Hyprland workspace reference embedded in monitor state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HyprlandWorkspaceRef {
    id: i64,
    name: String,
}

impl HyprlandWorkspaceRef {
    /// Returns the numeric Hyprland workspace id.
    #[must_use]
    pub const fn id(&self) -> i64 {
        self.id
    }

    /// Returns the Hyprland workspace name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Monitor position in Hyprland layout coordinates.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MonitorPosition {
    /// Horizontal coordinate.
    pub x: i32,
    /// Vertical coordinate.
    pub y: i32,
}

/// Parsed `hyprctl -j monitors` monitor entry.
#[derive(Clone, Debug, PartialEq)]
pub struct HyprlandMonitor {
    id: i64,
    name: String,
    description: String,
    make: String,
    model: String,
    width: u32,
    height: u32,
    physical_width: u32,
    physical_height: u32,
    refresh_rate: f64,
    x: i32,
    y: i32,
    active_workspace: HyprlandWorkspaceRef,
    scale: f64,
    focused: bool,
    dpms_status: bool,
    disabled: bool,
}

impl HyprlandMonitor {
    /// Returns the numeric Hyprland monitor id.
    #[must_use]
    pub const fn id(&self) -> i64 {
        self.id
    }

    /// Returns the monitor connector name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the sanitized display description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the monitor make string.
    #[must_use]
    pub fn make(&self) -> &str {
        &self.make
    }

    /// Returns the monitor model string.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Returns the active pixel width.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the active pixel height.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the physical width in millimeters.
    #[must_use]
    pub const fn physical_width(&self) -> u32 {
        self.physical_width
    }

    /// Returns the physical height in millimeters.
    #[must_use]
    pub const fn physical_height(&self) -> u32 {
        self.physical_height
    }

    /// Returns the refresh rate in hertz.
    #[must_use]
    pub const fn refresh_rate(&self) -> f64 {
        self.refresh_rate
    }

    /// Returns the layout position.
    #[must_use]
    pub const fn position(&self) -> MonitorPosition {
        MonitorPosition {
            x: self.x,
            y: self.y,
        }
    }

    /// Returns the active workspace reference.
    #[must_use]
    pub const fn active_workspace(&self) -> &HyprlandWorkspaceRef {
        &self.active_workspace
    }

    /// Returns the output scale.
    #[must_use]
    pub const fn scale(&self) -> f64 {
        self.scale
    }

    /// Returns whether Hyprland marks this monitor focused.
    #[must_use]
    pub const fn focused(&self) -> bool {
        self.focused
    }

    /// Returns whether DPMS is enabled for this monitor.
    #[must_use]
    pub const fn dpms_status(&self) -> bool {
        self.dpms_status
    }

    /// Returns whether this monitor is disabled.
    #[must_use]
    pub const fn disabled(&self) -> bool {
        self.disabled
    }
}

/// Parsed `hyprctl -j workspaces` workspace entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HyprlandWorkspace {
    id: i64,
    name: String,
    monitor: String,
    monitor_id: i64,
    windows: u32,
    hasfullscreen: bool,
    lastwindow: String,
    lastwindowtitle: String,
    ispersistent: bool,
    tiled_layout: String,
}

impl HyprlandWorkspace {
    /// Returns the numeric Hyprland workspace id.
    #[must_use]
    pub const fn id(&self) -> i64 {
        self.id
    }

    /// Returns the Hyprland workspace name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the owning monitor connector name.
    #[must_use]
    pub fn monitor(&self) -> &str {
        &self.monitor
    }

    /// Returns the owning monitor id.
    #[must_use]
    pub const fn monitor_id(&self) -> i64 {
        self.monitor_id
    }

    /// Returns the current window count.
    #[must_use]
    pub const fn windows(&self) -> u32 {
        self.windows
    }

    /// Returns whether the workspace has a fullscreen window.
    #[must_use]
    pub const fn has_fullscreen(&self) -> bool {
        self.hasfullscreen
    }

    /// Returns the sanitized last-window address or Hyprland's `0x0` sentinel.
    #[must_use]
    pub fn last_window(&self) -> &str {
        &self.lastwindow
    }

    /// Returns the sanitized last-window title.
    #[must_use]
    pub fn last_window_title(&self) -> &str {
        &self.lastwindowtitle
    }

    /// Returns whether the workspace is persistent.
    #[must_use]
    pub const fn is_persistent(&self) -> bool {
        self.ispersistent
    }

    /// Returns the workspace tiled layout name.
    #[must_use]
    pub fn tiled_layout(&self) -> &str {
        &self.tiled_layout
    }
}

/// Parses `hyprctl -j monitors` JSON.
///
/// # Errors
///
/// Returns [`crate::HyprlandErrorKind::InvalidJson`] when the payload is malformed.
pub fn parse_monitors(payload: &str) -> Result<Vec<HyprlandMonitor>> {
    parse_monitors_with_context(payload, CommandContext::parser("monitors"))
}

/// Parses `hyprctl -j workspaces` JSON.
///
/// # Errors
///
/// Returns [`crate::HyprlandErrorKind::InvalidJson`] when the payload is malformed.
pub fn parse_workspaces(payload: &str) -> Result<Vec<HyprlandWorkspace>> {
    parse_workspaces_with_context(payload, CommandContext::parser("workspaces"))
}

/// Parses `hyprctl -j activeworkspace` JSON.
///
/// # Errors
///
/// Returns [`crate::HyprlandErrorKind::InvalidJson`] when the payload is malformed.
pub fn parse_active_workspace(payload: &str) -> Result<HyprlandWorkspace> {
    parse_active_workspace_with_context(payload, CommandContext::parser("activeworkspace"))
}

/// Parses `hyprctl -j monitors` JSON with explicit command context.
///
/// # Errors
///
/// Returns [`crate::HyprlandErrorKind::InvalidJson`] when the payload is malformed.
pub fn parse_monitors_with_context(
    payload: &str,
    context: CommandContext,
) -> Result<Vec<HyprlandMonitor>> {
    parser::parse_monitors(payload, context)
}

/// Parses `hyprctl -j workspaces` JSON with explicit command context.
///
/// # Errors
///
/// Returns [`crate::HyprlandErrorKind::InvalidJson`] when the payload is malformed.
pub fn parse_workspaces_with_context(
    payload: &str,
    context: CommandContext,
) -> Result<Vec<HyprlandWorkspace>> {
    parser::parse_workspaces(payload, context)
}

/// Parses `hyprctl -j activeworkspace` JSON with explicit command context.
///
/// # Errors
///
/// Returns [`crate::HyprlandErrorKind::InvalidJson`] when the payload is malformed.
pub fn parse_active_workspace_with_context(
    payload: &str,
    context: CommandContext,
) -> Result<HyprlandWorkspace> {
    parser::parse_active_workspace(payload, context)
}
