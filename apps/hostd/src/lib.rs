#![doc = "Host daemon control helpers for madobe."]
#![forbid(unsafe_code)]

use madobe_compositor::{
    BindSession, BindingStatus, ColorDepth, CompositorAdapter, CompositorError, ConfigError,
    CreateOutput, Dimensions, IdentifierError, OutputConfig, OutputId, OutputMode, OutputState,
    OutputStatus, Position, RefreshRate, Scale, SessionId, WorkspaceId,
};
use madobe_protocol::MadobeHello;
use madobe_telemetry::bootstrap_event;
use std::error::Error;
use std::fmt::{self, Display};

/// Returns the deterministic M0 host status line.
#[must_use]
pub fn status_line() -> String {
    let hello = MadobeHello::new();
    let event = bootstrap_event();

    format!(
        "{} event={} ts={} status=ok",
        hello.identity(),
        event.name(),
        event.timestamp().as_unix_millis()
    )
}

/// Remote display lifecycle action handled by hostd.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisplayAction {
    /// List madobe-owned remote displays.
    Status,
    /// Create or adopt the named display with the default smoke configuration.
    Create {
        /// Display identifier.
        id: OutputId,
    },
    /// Park the named display as a warm spare.
    Park {
        /// Display identifier.
        id: OutputId,
    },
    /// Remove the named display.
    Remove {
        /// Display identifier.
        id: OutputId,
    },
    /// Bind a session/workspace to the named display.
    Bind {
        /// Display identifier.
        id: OutputId,
        /// Remote session identifier.
        session: SessionId,
        /// Workspace identifier.
        workspace: WorkspaceId,
    },
    /// Create, park, and remove the named display.
    Smoke {
        /// Display identifier.
        id: OutputId,
    },
}

/// Host control failure.
#[derive(Debug)]
pub enum HostControlError {
    /// Static smoke configuration is invalid.
    Config(ConfigError),
    /// Static smoke output identifier is invalid.
    Identifier(IdentifierError),
    /// The compositor adapter rejected the operation.
    Compositor(CompositorError),
}

impl Display for HostControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(formatter, "invalid display configuration: {error}"),
            Self::Identifier(error) => write!(formatter, "invalid display identifier: {error}"),
            Self::Compositor(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for HostControlError {}

impl From<ConfigError> for HostControlError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}

impl From<IdentifierError> for HostControlError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<CompositorError> for HostControlError {
    fn from(error: CompositorError) -> Self {
        Self::Compositor(error)
    }
}

/// Returns the default output id used by lifecycle smoke commands.
///
/// # Errors
///
/// Returns [`IdentifierError`] if the built-in id fails validation.
pub fn default_smoke_output_id() -> Result<OutputId, IdentifierError> {
    OutputId::new("madobe-cli-smoke")
}

/// Returns the default remote display configuration used by lifecycle smoke commands.
///
/// # Errors
///
/// Returns [`ConfigError`] if the built-in mode is invalid.
pub fn default_display_config() -> Result<OutputConfig, ConfigError> {
    Ok(OutputConfig::new(
        OutputMode::new(
            Dimensions::new(1280, 720)?,
            RefreshRate::from_millihertz(60_000)?,
        ),
        Scale::one(),
        Position {
            x: 50_000,
            y: 50_000,
        },
        ColorDepth::Eight,
    ))
}

/// Runs a display lifecycle action through the compositor adapter contract.
///
/// # Errors
///
/// Returns [`HostControlError`] when the built-in smoke configuration is invalid or the adapter
/// rejects the requested operation.
pub fn run_display_action(
    adapter: &mut impl CompositorAdapter,
    action: DisplayAction,
) -> Result<String, HostControlError> {
    match action {
        DisplayAction::Status => display_status(adapter).map_err(Into::into),
        DisplayAction::Create { id } => {
            let status = adapter.create_output(CreateOutput::new(id, default_display_config()?))?;
            Ok(render_operation_status("create", &status))
        }
        DisplayAction::Park { id } => {
            let status = adapter.park_output(&id)?;
            Ok(render_operation_status("park", &status))
        }
        DisplayAction::Remove { id } => {
            adapter.remove_output(&id)?;
            Ok(format!("display remove id={id} status=removed"))
        }
        DisplayAction::Bind {
            id,
            session,
            workspace,
        } => {
            let status = adapter.bind_session(BindSession::new(session, id, workspace))?;
            Ok(render_binding_status(&status))
        }
        DisplayAction::Smoke { id } => run_smoke(adapter, &id),
    }
}

fn display_status(adapter: &impl CompositorAdapter) -> Result<String, CompositorError> {
    let mut outputs = adapter.list_outputs()?;
    outputs.sort_by(|left, right| left.id().cmp(right.id()));

    let mut lines = vec![format!("display status count={}", outputs.len())];
    lines.extend(
        outputs
            .iter()
            .map(|status| render_status_line("output", status)),
    );

    Ok(lines.join("\n"))
}

fn run_smoke(
    adapter: &mut impl CompositorAdapter,
    id: &OutputId,
) -> Result<String, HostControlError> {
    let created =
        adapter.create_output(CreateOutput::new(id.clone(), default_display_config()?))?;
    let parked = adapter.park_output(id)?;
    adapter.remove_output(id)?;

    Ok(format!(
        "display smoke id={id} create={} park={} remove=removed",
        render_state(created.state()),
        render_state(parked.state())
    ))
}

fn render_operation_status(operation: &str, status: &OutputStatus) -> String {
    render_status_line(&format!("display {operation}"), status)
}

fn render_binding_status(status: &BindingStatus) -> String {
    format!(
        "display bind id={} session={} workspace={} status=bound",
        status.output(),
        status.session(),
        status.workspace()
    )
}

fn render_status_line(prefix: &str, status: &OutputStatus) -> String {
    let config = status.config();
    let mode = config.mode();
    let dimensions = mode.dimensions();
    let position = config.position();
    let scale = config.scale();
    let workspace = status
        .workspace()
        .map_or("-", madobe_compositor::WorkspaceId::as_str);

    format!(
        "{prefix} id={} state={} size={}x{} refresh_millihertz={} scale={}/{} position={}x{} color_depth={} workspace={}",
        status.id(),
        render_state(status.state()),
        dimensions.width(),
        dimensions.height(),
        mode.refresh_rate().as_millihertz(),
        scale.numerator(),
        scale.denominator(),
        position.x,
        position.y,
        render_color_depth(config.color_depth()),
        workspace
    )
}

const fn render_state(state: OutputState) -> &'static str {
    match state {
        OutputState::Ready => "ready",
        OutputState::Bound => "bound",
        OutputState::Parked => "parked",
    }
}

const fn render_color_depth(color_depth: ColorDepth) -> &'static str {
    match color_depth {
        ColorDepth::Eight => "8",
        ColorDepth::Ten => "10",
        ColorDepth::Twelve => "12",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DisplayAction, default_display_config, default_smoke_output_id, run_display_action,
        status_line,
    };
    use madobe_compositor::{
        BindSession, BindingStatus, CompositorAdapter, CompositorError, CreateOutput, ErrorKind,
        Operation, OutputConfig, OutputId, OutputState, OutputStatus, ReconcileReport,
        ReconcileState, Resource, Result, WorkspaceId,
    };
    use std::collections::BTreeMap;

    #[test]
    fn status_line_links_shared_crates() {
        assert_eq!(
            status_line(),
            "madobe 0.1.0 protocol=1 event=madobe.bootstrap ts=0 status=ok"
        );
    }

    #[test]
    fn display_status_is_sorted_and_deterministic() {
        let mut adapter = MockAdapter::with_outputs([
            (id("madobe-z"), OutputState::Parked, None),
            (
                id("madobe-a"),
                OutputState::Ready,
                Some(workspace("madobe-ws")),
            ),
        ]);

        assert_eq!(
            must(run_display_action(&mut adapter, DisplayAction::Status)),
            "display status count=2\noutput id=madobe-a state=ready size=1280x720 refresh_millihertz=60000 scale=1/1 position=50000x50000 color_depth=8 workspace=madobe-ws\noutput id=madobe-z state=parked size=1280x720 refresh_millihertz=60000 scale=1/1 position=50000x50000 color_depth=8 workspace=-"
        );
    }

    #[test]
    fn smoke_command_uses_adapter_contract_for_lifecycle() {
        let mut adapter = MockAdapter::default();
        let output = id("madobe-test-smoke");

        assert_eq!(
            must(run_display_action(
                &mut adapter,
                DisplayAction::Smoke { id: output },
            )),
            "display smoke id=madobe-test-smoke create=ready park=parked remove=removed"
        );
        assert!(adapter.outputs.is_empty());
        assert_eq!(
            adapter.calls,
            vec![
                "create:madobe-test-smoke",
                "park:madobe-test-smoke",
                "remove:madobe-test-smoke",
            ]
        );
    }

    #[test]
    fn lifecycle_commands_render_single_operation_results() {
        let mut adapter = MockAdapter::default();
        let output = id("madobe-test-output");

        assert_eq!(
            must(run_display_action(
                &mut adapter,
                DisplayAction::Create { id: output.clone() },
            )),
            "display create id=madobe-test-output state=ready size=1280x720 refresh_millihertz=60000 scale=1/1 position=50000x50000 color_depth=8 workspace=-"
        );
        assert_eq!(
            must(run_display_action(
                &mut adapter,
                DisplayAction::Park { id: output.clone() },
            )),
            "display park id=madobe-test-output state=parked size=1280x720 refresh_millihertz=60000 scale=1/1 position=50000x50000 color_depth=8 workspace=-"
        );
        assert_eq!(
            must(run_display_action(
                &mut adapter,
                DisplayAction::Remove { id: output },
            )),
            "display remove id=madobe-test-output status=removed"
        );
    }

    #[test]
    fn default_smoke_identifier_is_stable() {
        let action = DisplayAction::Status;

        assert_eq!(action, DisplayAction::Status);
        assert_eq!(must(default_smoke_output_id()).as_str(), "madobe-cli-smoke");
    }

    #[derive(Default)]
    struct MockAdapter {
        outputs: BTreeMap<OutputId, OutputStatus>,
        calls: Vec<String>,
    }

    impl MockAdapter {
        fn with_outputs<const N: usize>(
            outputs: [(OutputId, OutputState, Option<WorkspaceId>); N],
        ) -> Self {
            Self {
                outputs: outputs
                    .into_iter()
                    .map(|(id, state, workspace)| {
                        (
                            id.clone(),
                            OutputStatus::new(id, must(default_display_config()), state, workspace),
                        )
                    })
                    .collect(),
                calls: Vec::new(),
            }
        }
    }

    impl CompositorAdapter for MockAdapter {
        fn create_output(&mut self, request: CreateOutput) -> Result<OutputStatus> {
            self.calls.push(format!("create:{}", request.id()));
            let status = OutputStatus::new(
                request.id().clone(),
                request.config(),
                OutputState::Ready,
                None,
            );
            self.outputs.insert(request.id().clone(), status.clone());

            Ok(status)
        }

        fn configure_output(
            &mut self,
            id: &OutputId,
            config: OutputConfig,
        ) -> Result<OutputStatus> {
            self.calls.push(format!("configure:{id}"));
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
            self.calls.push(format!("park:{id}"));
            let status = self.outputs.get_mut(id).ok_or_else(|| missing(id))?;
            let updated = OutputStatus::new(id.clone(), status.config(), OutputState::Parked, None);
            *status = updated.clone();

            Ok(updated)
        }

        fn remove_output(&mut self, id: &OutputId) -> Result<()> {
            self.calls.push(format!("remove:{id}"));
            self.outputs.remove(id).ok_or_else(|| missing(id))?;

            Ok(())
        }

        fn list_outputs(&self) -> Result<Vec<OutputStatus>> {
            Ok(self.outputs.values().cloned().collect())
        }

        fn reconcile(&mut self, _desired: &ReconcileState) -> Result<ReconcileReport> {
            Err(CompositorError::new(
                Operation::Reconcile,
                ErrorKind::InvariantViolation {
                    reason: "mock reconcile not used".to_owned(),
                },
            ))
        }

        fn bind_session(&mut self, _request: BindSession) -> Result<BindingStatus> {
            Err(CompositorError::new(
                Operation::Bind,
                ErrorKind::InvariantViolation {
                    reason: "mock bind not used".to_owned(),
                },
            ))
        }
    }

    fn id(value: &str) -> OutputId {
        must(OutputId::new(value))
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

    fn must<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        }
    }
}
