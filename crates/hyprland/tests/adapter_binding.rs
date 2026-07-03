#![doc = "Hyprland adapter workspace binding tests."]
#![forbid(unsafe_code)]

use madobe_compositor::{BindSession, CompositorAdapter, OutputId, SessionId, WorkspaceId};
use madobe_hyprland::{CommandExecutor, CommandOutput, HyprctlCommand, HyprlandAdapter, Result};
use std::cell::RefCell;

#[test]
fn binds_workspace_to_named_output_with_lua_dispatcher() {
    let output = output("madobe-bind-output");
    let workspace = workspace("madobe-workspace-bound");
    let executor = BindExecutor::new(output.clone());
    let mut adapter = must(HyprlandAdapter::new(executor));

    let binding = must(adapter.bind_session(BindSession::new(
        session("madobe-session"),
        output.clone(),
        workspace.clone(),
    )));

    assert_eq!(binding.output(), &output);
    assert_eq!(binding.workspace(), &workspace);
    assert_eq!(
        adapter.client().executor().calls(),
        vec![
            args(["-j", "monitors"]),
            args([
                "eval",
                workspace_move_eval("madobe-workspace-bound", &output).as_str()
            ]),
        ]
    );
}

#[derive(Debug)]
struct BindExecutor {
    output: OutputId,
    calls: RefCell<Vec<Vec<String>>>,
}

impl BindExecutor {
    const fn new(output: OutputId) -> Self {
        Self {
            output,
            calls: RefCell::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<Vec<String>> {
        self.calls.borrow().clone()
    }
}

impl CommandExecutor for BindExecutor {
    fn execute(&self, command: &HyprctlCommand) -> Result<CommandOutput> {
        let args = command.args().to_vec();
        self.calls.borrow_mut().push(args.clone());

        if args == ["-j", "monitors"] {
            return Ok(CommandOutput::new(
                format!(
                    "[{{\"id\":1,\"name\":\"{}\",\"description\":\"madobe headless\",\"make\":\"madobe\",\"model\":\"remote\",\"width\":1280,\"height\":720,\"physicalWidth\":0,\"physicalHeight\":0,\"refreshRate\":60,\"x\":50000,\"y\":50000,\"activeWorkspace\":{{\"id\":101,\"name\":\"madobe-workspace\"}},\"scale\":1,\"focused\":false,\"dpmsStatus\":true,\"disabled\":false}}]",
                    self.output
                ),
                "",
            ));
        }

        if args.len() == 2 && args[0] == "eval" && args[1].contains("hl.dsp.workspace.move") {
            return Ok(CommandOutput::new("ok", ""));
        }

        panic!("unexpected command: {args:?}");
    }
}

fn output(value: &str) -> OutputId {
    must(OutputId::new(value))
}

fn session(value: &str) -> SessionId {
    must(SessionId::new(value))
}

fn workspace(value: &str) -> WorkspaceId {
    must(WorkspaceId::new(value))
}

fn workspace_move_eval(workspace: &str, output: &OutputId) -> String {
    format!(
        "hl.dispatch(hl.dsp.workspace.move({{ workspace = 'name:{workspace}', monitor = '{output}' }}))"
    )
}

fn args<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(str::to_owned).collect()
}

fn must<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error:?}"),
    }
}
