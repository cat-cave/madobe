#![doc = "Fixture-backed Hyprland command layer tests."]

use madobe_hyprland::{
    CommandExecutor, CommandOutput, EventSource, HyprctlCommand, HyprlandClient, HyprlandError,
    HyprlandErrorKind, LinesEventSource, Result, parse_event_line, parse_monitors,
    parse_workspaces,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Cursor;

const MONITORS: &str = include_str!("../../../fixtures/hyprland/monitors.json");
const WORKSPACES: &str = include_str!("../../../fixtures/hyprland/workspaces.json");
const ACTIVE_WORKSPACE: &str = include_str!("../../../fixtures/hyprland/activeworkspace.json");

#[test]
fn parses_monitor_fixture() {
    let monitors = must(parse_monitors(MONITORS));

    assert_eq!(monitors.len(), 2);
    assert_eq!(monitors[0].id(), 1);
    assert_eq!(monitors[0].name(), "DP-2");
    assert_eq!(monitors[0].make(), "LG Electronics");
    assert_eq!(monitors[0].model(), "LG ULTRAGEAR+");
    assert_eq!(monitors[0].width(), 3840);
    assert_eq!(monitors[0].height(), 2160);
    assert_eq!(monitors[0].physical_width(), 600);
    assert_eq!(monitors[0].physical_height(), 340);
    assert_eq!(monitors[0].position().x, 0);
    assert_eq!(monitors[0].position().y, 0);
    assert_eq!(monitors[0].active_workspace().id(), 11);
    assert_eq!(monitors[0].active_workspace().name(), "workspace-11");
    assert!(monitors[0].focused());
    assert!(monitors[0].dpms_status());
    assert!(!monitors[0].disabled());

    assert_eq!(monitors[1].id(), 2);
    assert_eq!(monitors[1].name(), "DP-3");
    assert_eq!(monitors[1].position().x, 3840);
    assert_eq!(monitors[1].active_workspace().id(), 12);
    assert!(!monitors[1].focused());
}

#[test]
fn parses_workspace_fixtures() {
    let workspaces = must(parse_workspaces(WORKSPACES));

    assert_eq!(workspaces.len(), 2);
    assert_eq!(workspaces[0].id(), 11);
    assert_eq!(workspaces[0].name(), "workspace-11");
    assert_eq!(workspaces[0].monitor(), "DP-2");
    assert_eq!(workspaces[0].monitor_id(), 1);
    assert_eq!(workspaces[0].windows(), 2);
    assert!(!workspaces[0].has_fullscreen());
    assert_eq!(workspaces[0].last_window(), "<redacted-window-address>");
    assert_eq!(workspaces[0].last_window_title(), "<redacted-window-title>");
    assert!(!workspaces[0].is_persistent());
    assert_eq!(workspaces[0].tiled_layout(), "dwindle");

    assert_eq!(workspaces[1].id(), 12);
    assert_eq!(workspaces[1].monitor(), "DP-3");
    assert_eq!(workspaces[1].last_window(), "0x0");
}

#[test]
fn client_uses_injected_executor_for_json_endpoints() {
    let executor = FixtureExecutor::new([
        (["-j", "monitors"], MONITORS),
        (["-j", "workspaces"], WORKSPACES),
        (["-j", "activeworkspace"], ACTIVE_WORKSPACE),
    ]);
    let client = HyprlandClient::new(executor);

    assert_eq!(must(client.monitors()).len(), 2);
    assert_eq!(must(client.workspaces()).len(), 2);
    assert_eq!(must(client.active_workspace()).id(), 11);

    assert_eq!(
        client.executor().calls(),
        vec![
            vec!["-j".to_owned(), "monitors".to_owned()],
            vec!["-j".to_owned(), "workspaces".to_owned()],
            vec!["-j".to_owned(), "activeworkspace".to_owned()],
        ]
    );
}

#[test]
fn parse_errors_preserve_command_context() {
    let executor = FixtureExecutor::new([(["-j", "workspaces"], "not-json")]);
    let client = HyprlandClient::new(executor);

    let error = match client.workspaces() {
        Ok(workspaces) => panic!("expected parse error, got {workspaces:?}"),
        Err(error) => error,
    };

    assert_eq!(error.context().program(), "hyprctl");
    assert_eq!(error.context().args(), ["-j", "workspaces"]);
    assert!(matches!(
        error.kind(),
        HyprlandErrorKind::InvalidJson { message } if message.contains("expected")
    ));
}

#[test]
fn parses_socket_event_lines() {
    let event = must(parse_event_line("workspace>>workspace-11\n"));
    assert_eq!(event.name(), "workspace");
    assert_eq!(event.payload(), "workspace-11");

    let mut source = LinesEventSource::new(Cursor::new("monitoradded>>DP-4\n"));
    let Some(sourced) = must(source.next_event()) else {
        panic!("expected one event");
    };
    assert_eq!(sourced.name(), "monitoradded");
    assert_eq!(sourced.payload(), "DP-4");
    assert_eq!(must(source.next_event()), None);
}

#[derive(Debug)]
struct FixtureExecutor {
    responses: BTreeMap<Vec<String>, &'static str>,
    calls: RefCell<Vec<Vec<String>>>,
}

impl FixtureExecutor {
    fn new<const N: usize>(responses: [([&'static str; 2], &'static str); N]) -> Self {
        Self {
            responses: responses
                .into_iter()
                .map(|(args, output)| {
                    (
                        args.into_iter().map(str::to_owned).collect::<Vec<_>>(),
                        output,
                    )
                })
                .collect(),
            calls: RefCell::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<Vec<String>> {
        self.calls.borrow().clone()
    }
}

impl CommandExecutor for FixtureExecutor {
    fn execute(&self, command: &HyprctlCommand) -> Result<CommandOutput> {
        let args = command.args().to_vec();
        self.calls.borrow_mut().push(args.clone());
        let Some(output) = self.responses.get(&args) else {
            return Err(HyprlandError::new(
                command.context(),
                HyprlandErrorKind::NonZeroExit {
                    code: Some(1),
                    stderr: "unexpected command".to_owned(),
                },
            ));
        };

        Ok(CommandOutput::new(*output, ""))
    }
}

fn must<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error:?}"),
    }
}
