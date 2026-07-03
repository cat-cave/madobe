use crate::{CommandExecutor, HyprctlCommand, HyprlandClient, HyprlandErrorKind};
use madobe_compositor::{
    BindSession, BindingStatus, Capability, ColorDepth, CompositorAdapter, CompositorError,
    ConfigError, CreateOutput, Dimensions, ErrorKind, Operation, OutputConfig, OutputId,
    OutputMode, OutputState, OutputStatus, Position, ReconcileReport, ReconcileState, RefreshRate,
    Resource, Scale, WorkspaceId,
};
use std::collections::BTreeSet;

mod rule;

pub use rule::monitor_rule;
use rule::{config_from_monitor, monitor_eval};

const OWNED_OUTPUT_PREFIX: &str = "madobe-";
const HEADLESS_BACKEND: &str = "headless";

/// Hyprland compositor adapter for madobe-owned headless outputs.
#[derive(Clone, Debug)]
pub struct HyprlandAdapter<E> {
    client: HyprlandClient<E>,
    parking_config: OutputConfig,
}

impl<E> HyprlandAdapter<E> {
    /// Creates an adapter from a command executor and the default parking configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if the built-in parking mode is invalid.
    pub fn new(executor: E) -> std::result::Result<Self, ConfigError> {
        Ok(Self::with_client(
            HyprlandClient::new(executor),
            default_parking_config()?,
        ))
    }

    /// Creates an adapter from a Hyprland client and explicit parking configuration.
    #[must_use]
    pub const fn with_client(client: HyprlandClient<E>, parking_config: OutputConfig) -> Self {
        Self {
            client,
            parking_config,
        }
    }

    /// Returns the wrapped Hyprland client.
    #[must_use]
    pub const fn client(&self) -> &HyprlandClient<E> {
        &self.client
    }

    /// Returns the output configuration used for parked warm spares.
    #[must_use]
    pub const fn parking_config(&self) -> OutputConfig {
        self.parking_config
    }
}

impl<E> CompositorAdapter for HyprlandAdapter<E>
where
    E: CommandExecutor,
{
    fn create_output(&mut self, request: CreateOutput) -> Result<OutputStatus, CompositorError> {
        ensure_owned(request.id(), Operation::Create)?;
        ensure_supported_config(request.config(), Operation::Create)?;

        if self.monitor_exists(request.id(), Operation::Create)? {
            return self.configure_output(request.id(), request.config());
        }

        self.run(
            &HyprctlCommand::new(["output", "create", HEADLESS_BACKEND, request.id().as_str()])
                .map_err(|error| map_hyprland_error(Operation::Create, &error))?,
            Operation::Create,
        )?;

        self.run_monitor_rule(request.id(), request.config(), Operation::Create)?;

        self.verified_status(
            request.id(),
            request.config(),
            OutputState::Ready,
            Operation::Create,
        )
    }

    fn configure_output(
        &mut self,
        id: &OutputId,
        config: OutputConfig,
    ) -> Result<OutputStatus, CompositorError> {
        ensure_owned(id, Operation::Configure)?;
        ensure_supported_config(config, Operation::Configure)?;
        ensure_exists(
            self.monitor_exists(id, Operation::Configure)?,
            id,
            Operation::Configure,
        )?;

        self.run_monitor_rule(id, config, Operation::Configure)?;

        self.verified_status(id, config, OutputState::Ready, Operation::Configure)
    }

    fn park_output(&mut self, id: &OutputId) -> Result<OutputStatus, CompositorError> {
        ensure_owned(id, Operation::Park)?;
        ensure_exists(
            self.monitor_exists(id, Operation::Park)?,
            id,
            Operation::Park,
        )?;

        self.run_monitor_rule(id, self.parking_config, Operation::Park)?;

        self.verified_status(
            id,
            self.parking_config,
            OutputState::Parked,
            Operation::Park,
        )
    }

    fn remove_output(&mut self, id: &OutputId) -> Result<(), CompositorError> {
        ensure_owned(id, Operation::Remove)?;
        if !self.monitor_exists(id, Operation::Remove)? {
            return Ok(());
        }

        let command = HyprctlCommand::new(["output", "remove", id.as_str()])
            .map_err(|error| map_hyprland_error(Operation::Remove, &error))?;

        match self.client.run(&command) {
            Ok(_) => Ok(()),
            Err(error) if is_missing_output_error(&error) => Ok(()),
            Err(error) => Err(map_hyprland_error(Operation::Remove, &error)),
        }
    }

    fn list_outputs(&self) -> Result<Vec<OutputStatus>, CompositorError> {
        self.list_owned_outputs(Operation::List)
    }

    fn reconcile(&mut self, desired: &ReconcileState) -> Result<ReconcileReport, CompositorError> {
        let mut report = ReconcileReport::default();
        let mut desired_ids = BTreeSet::new();

        for output in desired.outputs() {
            ensure_owned(output.id(), Operation::Reconcile)?;
            ensure_supported_config(output.config(), Operation::Reconcile)?;
            desired_ids.insert(output.id().clone());
        }

        let current = self.list_owned_outputs(Operation::Reconcile)?;
        let current_ids = current
            .iter()
            .map(|status| status.id().clone())
            .collect::<BTreeSet<_>>();

        for output in desired.outputs() {
            if current_ids.contains(output.id()) {
                let current_status = current
                    .iter()
                    .find(|status| status.id() == output.id())
                    .ok_or_else(|| invariant(Operation::Reconcile, "current output disappeared"))?;

                if current_status.config() != output.config()
                    || current_status.state() == OutputState::Parked
                {
                    self.configure_output(output.id(), output.config())?;
                    report.configured += 1;
                }
            } else {
                self.create_output(output.clone())?;
                report.created += 1;
            }
        }

        for status in current {
            if !desired_ids.contains(status.id()) {
                self.park_output(status.id())?;
                report.parked += 1;
            }
        }

        for binding in desired.bindings() {
            self.bind_session(binding.clone())?;
            report.bound += 1;
        }

        Ok(report)
    }

    fn bind_session(&mut self, request: BindSession) -> Result<BindingStatus, CompositorError> {
        ensure_owned(request.output(), Operation::Bind)?;
        ensure_exists(
            self.monitor_exists(request.output(), Operation::Bind)?,
            request.output(),
            Operation::Bind,
        )?;

        let workspace = workspace_ref(request.workspace());
        self.run(
            &HyprctlCommand::new([
                "dispatch",
                "moveworkspacetomonitor",
                workspace.as_str(),
                request.output().as_str(),
            ])
            .map_err(|error| map_hyprland_error(Operation::Bind, &error))?,
            Operation::Bind,
        )?;

        Ok(BindingStatus::new(
            request.session().clone(),
            request.output().clone(),
            request.workspace().clone(),
        ))
    }
}

impl<E> HyprlandAdapter<E>
where
    E: CommandExecutor,
{
    fn run(&self, command: &HyprctlCommand, operation: Operation) -> Result<(), CompositorError> {
        self.client
            .run(command)
            .map(|_| ())
            .map_err(|error| map_hyprland_error(operation, &error))
    }

    fn run_monitor_rule(
        &self,
        id: &OutputId,
        config: OutputConfig,
        operation: Operation,
    ) -> Result<(), CompositorError> {
        self.run(
            &HyprctlCommand::new(["eval", monitor_eval(id, config).as_str()])
                .map_err(|error| map_hyprland_error(operation, &error))?,
            operation,
        )
    }

    fn monitor_exists(&self, id: &OutputId, operation: Operation) -> Result<bool, CompositorError> {
        Ok(self
            .client
            .monitors()
            .map_err(|error| map_hyprland_error(operation, &error))?
            .iter()
            .any(|monitor| monitor.name() == id.as_str()))
    }

    fn verified_status(
        &self,
        id: &OutputId,
        expected_config: OutputConfig,
        expected_state: OutputState,
        operation: Operation,
    ) -> Result<OutputStatus, CompositorError> {
        let Some(status) = self
            .list_owned_outputs(operation)?
            .into_iter()
            .find(|status| status.id() == id)
        else {
            return Err(missing_output(id, operation));
        };

        if status.config() != expected_config {
            return Err(invariant(
                operation,
                "Hyprland accepted monitor rule but state did not change",
            ));
        }

        Ok(OutputStatus::new(
            id.clone(),
            expected_config,
            expected_state,
            status.workspace().cloned(),
        ))
    }

    fn list_owned_outputs(
        &self,
        operation: Operation,
    ) -> Result<Vec<OutputStatus>, CompositorError> {
        self.client
            .monitors()
            .map_err(|error| map_hyprland_error(operation, &error))?
            .into_iter()
            .filter(|monitor| monitor.name().starts_with(OWNED_OUTPUT_PREFIX))
            .map(|monitor| {
                let id = OutputId::new(monitor.name()).map_err(|error| {
                    CompositorError::new(
                        operation,
                        ErrorKind::InvariantViolation {
                            reason: format!("invalid owned output id from Hyprland: {error}"),
                        },
                    )
                })?;
                let config = config_from_monitor(&monitor, operation)?;
                let state = if config == self.parking_config {
                    OutputState::Parked
                } else {
                    OutputState::Ready
                };
                let workspace = WorkspaceId::new(monitor.active_workspace().name()).ok();

                Ok(OutputStatus::new(id, config, state, workspace))
            })
            .collect()
    }
}

fn ensure_owned(id: &OutputId, operation: Operation) -> Result<(), CompositorError> {
    if id.as_str().starts_with(OWNED_OUTPUT_PREFIX) {
        return Ok(());
    }

    Err(CompositorError::new(
        operation,
        ErrorKind::InvalidConfig {
            reason: format!("Hyprland adapter only manages {OWNED_OUTPUT_PREFIX} outputs"),
        },
    ))
}

fn ensure_exists(exists: bool, id: &OutputId, operation: Operation) -> Result<(), CompositorError> {
    if exists {
        return Ok(());
    }

    Err(missing_output(id, operation))
}

fn missing_output(id: &OutputId, operation: Operation) -> CompositorError {
    CompositorError::new(
        operation,
        ErrorKind::NotFound {
            resource: Resource::Output,
            id: id.to_string(),
        },
    )
}

fn ensure_supported_config(
    config: OutputConfig,
    operation: Operation,
) -> Result<(), CompositorError> {
    if config.color_depth() == ColorDepth::Eight {
        return Ok(());
    }

    Err(CompositorError::new(
        operation,
        ErrorKind::Unsupported {
            capability: Capability::ColorDepth,
        },
    ))
}

fn default_parking_config() -> std::result::Result<OutputConfig, ConfigError> {
    Ok(OutputConfig::new(
        OutputMode::new(
            Dimensions::new(640, 480)?,
            RefreshRate::from_millihertz(30_000)?,
        ),
        Scale::one(),
        Position {
            x: 50_000,
            y: 50_000,
        },
        ColorDepth::Eight,
    ))
}

fn workspace_ref(workspace: &WorkspaceId) -> String {
    format!("name:{workspace}")
}

fn is_missing_output_error(error: &crate::HyprlandError) -> bool {
    match error.kind() {
        HyprlandErrorKind::NonZeroExit { stderr, .. } => {
            let stderr = stderr.to_ascii_lowercase();
            stderr.contains("not found")
                || stderr.contains("does not exist")
                || stderr.contains("doesn't exist")
                || stderr.contains("no such")
        }
        _ => false,
    }
}

fn map_hyprland_error(operation: Operation, error: &crate::HyprlandError) -> CompositorError {
    match error.kind() {
        HyprlandErrorKind::Spawn { message } => CompositorError::new(
            operation,
            ErrorKind::Unavailable {
                reason: message.clone(),
            },
        ),
        HyprlandErrorKind::NonZeroExit { stderr, .. }
            if stderr.to_ascii_lowercase().contains("permission") =>
        {
            CompositorError::new(
                operation,
                ErrorKind::PermissionDenied {
                    reason: stderr.clone(),
                },
            )
        }
        HyprlandErrorKind::NonZeroExit { stderr, .. } => CompositorError::new(
            operation,
            ErrorKind::Unavailable {
                reason: stderr.clone(),
            },
        ),
        HyprlandErrorKind::InvalidCommand { reason } => CompositorError::new(
            operation,
            ErrorKind::InvalidConfig {
                reason: reason.clone(),
            },
        ),
        HyprlandErrorKind::InvalidJson { message }
        | HyprlandErrorKind::InvalidUtf8 { message }
        | HyprlandErrorKind::EventIo { message } => CompositorError::new(
            operation,
            ErrorKind::InvariantViolation {
                reason: message.clone(),
            },
        ),
        HyprlandErrorKind::InvalidEvent { reason } => CompositorError::new(
            operation,
            ErrorKind::InvariantViolation {
                reason: reason.clone(),
            },
        ),
    }
}

fn invariant(operation: Operation, reason: &str) -> CompositorError {
    CompositorError::new(
        operation,
        ErrorKind::InvariantViolation {
            reason: reason.to_owned(),
        },
    )
}
