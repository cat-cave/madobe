#![doc = "Host display binding integration tests."]
#![forbid(unsafe_code)]

use hostd::{DisplayAction, run_display_action};
use madobe_compositor::{
    BindSession, BindingStatus, CompositorAdapter, CompositorError, CreateOutput, ErrorKind,
    Operation, OutputConfig, OutputId, OutputState, OutputStatus, ReconcileReport, ReconcileState,
    Resource, Result, SessionId, WorkspaceId,
};
use std::collections::BTreeMap;

#[test]
fn bind_command_renders_session_workspace_result() {
    let mut adapter = BindAdapter::with_output("madobe-test-output");

    assert_eq!(
        must(run_display_action(
            &mut adapter,
            DisplayAction::Bind {
                id: id("madobe-test-output"),
                session: session("madobe-test-session"),
                workspace: workspace("madobe-test-workspace"),
            },
        )),
        "display bind id=madobe-test-output session=madobe-test-session workspace=madobe-test-workspace status=bound"
    );
    assert_eq!(
        adapter.calls,
        vec!["bind:madobe-test-session:madobe-test-output:madobe-test-workspace"]
    );
}

#[derive(Default)]
struct BindAdapter {
    outputs: BTreeMap<OutputId, OutputStatus>,
    calls: Vec<String>,
}

impl BindAdapter {
    fn with_output(value: &str) -> Self {
        let output = id(value);
        let status = OutputStatus::new(
            output.clone(),
            must(hostd::default_display_config()),
            OutputState::Ready,
            None,
        );

        Self {
            outputs: BTreeMap::from([(output, status)]),
            calls: Vec::new(),
        }
    }
}

impl CompositorAdapter for BindAdapter {
    fn create_output(&mut self, _request: CreateOutput) -> Result<OutputStatus> {
        Err(unused(Operation::Create))
    }

    fn configure_output(&mut self, _id: &OutputId, _config: OutputConfig) -> Result<OutputStatus> {
        Err(unused(Operation::Configure))
    }

    fn park_output(&mut self, _id: &OutputId) -> Result<OutputStatus> {
        Err(unused(Operation::Park))
    }

    fn remove_output(&mut self, _id: &OutputId) -> Result<()> {
        Err(unused(Operation::Remove))
    }

    fn list_outputs(&self) -> Result<Vec<OutputStatus>> {
        Ok(self.outputs.values().cloned().collect())
    }

    fn reconcile(&mut self, _desired: &ReconcileState) -> Result<ReconcileReport> {
        Err(unused(Operation::Reconcile))
    }

    fn bind_session(&mut self, request: BindSession) -> Result<BindingStatus> {
        self.calls.push(format!(
            "bind:{}:{}:{}",
            request.session(),
            request.output(),
            request.workspace()
        ));
        let status = self
            .outputs
            .get_mut(request.output())
            .ok_or_else(|| missing(request.output()))?;
        *status = OutputStatus::new(
            request.output().clone(),
            status.config(),
            OutputState::Bound,
            Some(request.workspace().clone()),
        );

        Ok(BindingStatus::new(
            request.session().clone(),
            request.output().clone(),
            request.workspace().clone(),
        ))
    }
}

fn id(value: &str) -> OutputId {
    must(OutputId::new(value))
}

fn session(value: &str) -> SessionId {
    must(SessionId::new(value))
}

fn workspace(value: &str) -> WorkspaceId {
    must(WorkspaceId::new(value))
}

fn missing(id: &OutputId) -> CompositorError {
    CompositorError::new(
        Operation::List,
        ErrorKind::NotFound {
            resource: Resource::Output,
            id: id.to_string(),
        },
    )
}

fn unused(operation: Operation) -> CompositorError {
    CompositorError::new(
        operation,
        ErrorKind::InvariantViolation {
            reason: "bind test called an unrelated adapter operation".to_owned(),
        },
    )
}

fn must<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error:?}"),
    }
}
