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

#[cfg(test)]
mod tests {
    use super::{parse_monitors, parse_workspaces};

    const ACCESSOR_MONITORS: &str = r#"[
        {
            "id": 31,
            "name": "HEADLESS-31",
            "description": "Madobe virtual display A",
            "make": "madobe",
            "model": "remote-a",
            "width": 1600,
            "height": 900,
            "physicalWidth": 0,
            "physicalHeight": 0,
            "refreshRate": 60,
            "x": 0,
            "y": 0,
            "activeWorkspace": {
                "id": 41,
                "name": "workspace-41"
            },
            "scale": 1.25,
            "focused": false,
            "dpmsStatus": false,
            "disabled": true
        },
        {
            "id": 32,
            "name": "HEADLESS-32",
            "description": "Madobe virtual display B",
            "make": "madobe",
            "model": "remote-b",
            "width": 1920,
            "height": 1080,
            "physicalWidth": 0,
            "physicalHeight": 0,
            "refreshRate": 144,
            "x": 1600,
            "y": 0,
            "activeWorkspace": {
                "id": 42,
                "name": "workspace-42"
            },
            "scale": 0.75,
            "focused": true,
            "dpmsStatus": true,
            "disabled": false
        }
    ]"#;

    const ACCESSOR_WORKSPACES: &str = r#"[
        {
            "id": 41,
            "name": "workspace-41",
            "monitor": "HEADLESS-31",
            "monitorID": 31,
            "windows": 1,
            "hasfullscreen": true,
            "lastwindow": "0x123",
            "lastwindowtitle": "fullscreen window",
            "ispersistent": true,
            "tiledLayout": "master"
        },
        {
            "id": 42,
            "name": "workspace-42",
            "monitor": "HEADLESS-32",
            "monitorID": 32,
            "windows": 0,
            "hasfullscreen": false,
            "lastwindow": "0x0",
            "lastwindowtitle": "",
            "ispersistent": false,
            "tiledLayout": "dwindle"
        }
    ]"#;

    #[test]
    fn parsed_monitor_accessors_return_description_scale_dpms_and_disabled() {
        let monitors = must(parse_monitors(ACCESSOR_MONITORS));

        assert_eq!(monitors.len(), 2);
        assert_eq!(monitors[0].description(), "Madobe virtual display A");
        assert!((monitors[0].scale() - 1.25).abs() < f64::EPSILON);
        assert!(!monitors[0].dpms_status());
        assert!(monitors[0].disabled());

        assert_eq!(monitors[1].description(), "Madobe virtual display B");
        assert!((monitors[1].scale() - 0.75).abs() < f64::EPSILON);
        assert!(monitors[1].dpms_status());
        assert!(!monitors[1].disabled());
    }

    #[test]
    fn parsed_workspace_accessors_return_monitor_id_fullscreen_and_persistent() {
        let workspaces = must(parse_workspaces(ACCESSOR_WORKSPACES));

        assert_eq!(workspaces.len(), 2);
        assert_eq!(workspaces[0].monitor_id(), 31);
        assert!(workspaces[0].has_fullscreen());
        assert!(workspaces[0].is_persistent());

        assert_eq!(workspaces[1].monitor_id(), 32);
        assert!(!workspaces[1].has_fullscreen());
        assert!(!workspaces[1].is_persistent());
    }

    fn must<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        }
    }
}
