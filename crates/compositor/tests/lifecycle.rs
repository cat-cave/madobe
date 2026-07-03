#![doc = "Lifecycle coverage for the compositor adapter contract."]
#![forbid(unsafe_code)]

use madobe_compositor::{
    BindSession, BindingStatus, ColorDepth, CompositorAdapter, CompositorError, ConfigError,
    CreateOutput, Dimensions, ErrorKind, Operation, OutputConfig, OutputId, OutputMode,
    OutputState, OutputStatus, Position, ReconcileReport, ReconcileState, RefreshRate, Resource,
    Result, Scale, SessionId, WorkspaceId,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct MockAdapter {
    outputs: BTreeMap<OutputId, OutputStatus>,
    bindings: BTreeSet<(SessionId, OutputId, WorkspaceId)>,
}

impl CompositorAdapter for MockAdapter {
    fn create_output(&mut self, request: CreateOutput) -> Result<OutputStatus> {
        if self.outputs.contains_key(request.id()) {
            return Err(error(
                Operation::Create,
                ErrorKind::AlreadyExists {
                    resource: Resource::Output,
                    id: request.id().to_string(),
                },
            ));
        }

        let status = OutputStatus::new(
            request.id().clone(),
            request.config(),
            OutputState::Ready,
            None,
        );
        self.outputs.insert(request.id().clone(), status.clone());

        Ok(status)
    }

    fn configure_output(&mut self, id: &OutputId, config: OutputConfig) -> Result<OutputStatus> {
        let status = self.outputs.get_mut(id).ok_or_else(|| missing(id))?;
        let updated = OutputStatus::new(
            id.clone(),
            config,
            status.state(),
            status.workspace().cloned(),
        );

        *status = updated.clone();

        Ok(updated)
    }

    fn park_output(&mut self, id: &OutputId) -> Result<OutputStatus> {
        let status = self.outputs.get_mut(id).ok_or_else(|| missing(id))?;
        let updated = OutputStatus::new(id.clone(), status.config(), OutputState::Parked, None);

        *status = updated.clone();

        Ok(updated)
    }

    fn remove_output(&mut self, id: &OutputId) -> Result<()> {
        self.outputs.remove(id).ok_or_else(|| missing(id))?;
        self.bindings.retain(|(_, output, _)| output != id);

        Ok(())
    }

    fn list_outputs(&self) -> Result<Vec<OutputStatus>> {
        Ok(self.outputs.values().cloned().collect())
    }

    fn reconcile(&mut self, desired: &ReconcileState) -> Result<ReconcileReport> {
        let mut report = ReconcileReport::default();

        for output in desired.outputs() {
            if self.outputs.contains_key(output.id()) {
                self.configure_output(output.id(), output.config())?;
                report.configured += 1;
            } else {
                self.create_output(output.clone())?;
                report.created += 1;
            }
        }

        let desired_outputs: BTreeSet<&OutputId> =
            desired.outputs().iter().map(CreateOutput::id).collect();
        let current_outputs: Vec<OutputId> = self.outputs.keys().cloned().collect();

        for output in current_outputs {
            if !desired_outputs.contains(&output) {
                self.park_output(&output)?;
                report.parked += 1;
            }
        }

        for binding in desired.bindings() {
            self.bind_session(binding.clone())?;
            report.bound += 1;
        }

        Ok(report)
    }

    fn bind_session(&mut self, request: BindSession) -> Result<BindingStatus> {
        let status = self
            .outputs
            .get_mut(request.output())
            .ok_or_else(|| missing(request.output()))?;
        let updated = OutputStatus::new(
            request.output().clone(),
            status.config(),
            OutputState::Bound,
            Some(request.workspace().clone()),
        );

        *status = updated;
        self.bindings.insert((
            request.session().clone(),
            request.output().clone(),
            request.workspace().clone(),
        ));

        Ok(BindingStatus::new(
            request.session().clone(),
            request.output().clone(),
            request.workspace().clone(),
        ))
    }
}

#[test]
fn lifecycle_supports_create_configure_bind_park_remove_list_and_reconcile() {
    let mut adapter = MockAdapter::default();
    let output = id("madobe-1");
    let session = session("session-1");
    let workspace = workspace("madobe-workspace-1");
    let initial = config(1920, 1080, 60_000);
    let updated = config(2560, 1440, 144_000);

    let created = must(
        adapter.create_output(CreateOutput::new(output.clone(), initial)),
        "mock creates output",
    );

    assert_eq!(created.state(), OutputState::Ready);
    assert_eq!(created.config(), initial);

    let configured = must(
        adapter.configure_output(&output, updated),
        "mock configures output",
    );

    assert_eq!(configured.config(), updated);

    let binding = must(
        adapter.bind_session(BindSession::new(
            session.clone(),
            output.clone(),
            workspace.clone(),
        )),
        "mock binds session",
    );

    assert_eq!(binding.session(), &session);
    assert_eq!(binding.output(), &output);
    assert_eq!(binding.workspace(), &workspace);
    assert_eq!(must(adapter.list_outputs(), "mock lists outputs").len(), 1);

    let parked = must(adapter.park_output(&output), "mock parks output");

    assert_eq!(parked.state(), OutputState::Parked);
    assert_eq!(parked.workspace(), None);

    must(adapter.remove_output(&output), "mock removes output");
    assert!(must(adapter.list_outputs(), "mock lists outputs").is_empty());

    let mut desired = ReconcileState::new();
    desired.add_output(CreateOutput::new(output.clone(), initial));
    desired.add_binding(BindSession::new(session, output.clone(), workspace));

    let report = must(adapter.reconcile(&desired), "mock reconciles output");

    assert_eq!(
        report,
        ReconcileReport {
            created: 1,
            configured: 0,
            parked: 0,
            removed: 0,
            bound: 1,
        }
    );
    assert_eq!(
        must(adapter.list_outputs(), "mock lists outputs")[0].state(),
        OutputState::Bound
    );
}

#[test]
fn typed_validation_errors_reject_invalid_values() {
    assert_eq!(
        OutputId::new(""),
        Err(madobe_compositor::IdentifierError::Empty)
    );
    assert_eq!(Dimensions::new(0, 1), Err(ConfigError::EmptyDimensions));
    assert_eq!(
        RefreshRate::from_millihertz(0),
        Err(ConfigError::MissingRefreshRate)
    );
    assert_eq!(Scale::new(1, 0), Err(ConfigError::InvalidScaleRatio));
}

const fn error(operation: Operation, kind: ErrorKind) -> CompositorError {
    CompositorError::new(operation, kind)
}

fn missing(id: &OutputId) -> CompositorError {
    error(
        Operation::List,
        ErrorKind::NotFound {
            resource: Resource::Output,
            id: id.to_string(),
        },
    )
}

fn id(value: &str) -> OutputId {
    must(OutputId::new(value), "valid output id")
}

fn session(value: &str) -> SessionId {
    must(SessionId::new(value), "valid session id")
}

fn workspace(value: &str) -> WorkspaceId {
    must(WorkspaceId::new(value), "valid workspace id")
}

fn config(width: u32, height: u32, millihertz: u32) -> OutputConfig {
    OutputConfig::new(
        OutputMode::new(
            must(Dimensions::new(width, height), "valid dimensions"),
            must(
                RefreshRate::from_millihertz(millihertz),
                "valid refresh rate",
            ),
        ),
        Scale::one(),
        Position { x: 50_000, y: 0 },
        ColorDepth::Eight,
    )
}

fn must<T, E: std::fmt::Debug>(result: std::result::Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error:?}"),
    }
}
