use crate::{OutputConfig, OutputId, SessionId, WorkspaceId};

/// Output lifecycle state visible to callers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputState {
    /// The output is allocated but not bound to a live session.
    Ready,
    /// The output is active for a bound session.
    Bound,
    /// The output is retained as a warm spare.
    Parked,
}

/// Current compositor view of an output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputStatus {
    id: OutputId,
    config: OutputConfig,
    state: OutputState,
    workspace: Option<WorkspaceId>,
}

impl OutputStatus {
    /// Creates an output status value.
    #[must_use]
    pub const fn new(
        id: OutputId,
        config: OutputConfig,
        state: OutputState,
        workspace: Option<WorkspaceId>,
    ) -> Self {
        Self {
            id,
            config,
            state,
            workspace,
        }
    }

    /// Returns the output identifier.
    #[must_use]
    pub const fn id(&self) -> &OutputId {
        &self.id
    }

    /// Returns the current output configuration.
    #[must_use]
    pub const fn config(&self) -> OutputConfig {
        self.config
    }

    /// Returns the current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> OutputState {
        self.state
    }

    /// Returns the bound workspace, when present.
    #[must_use]
    pub const fn workspace(&self) -> Option<&WorkspaceId> {
        self.workspace.as_ref()
    }
}

/// Request to create a remote output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateOutput {
    id: OutputId,
    config: OutputConfig,
}

impl CreateOutput {
    /// Creates an output creation request.
    #[must_use]
    pub const fn new(id: OutputId, config: OutputConfig) -> Self {
        Self { id, config }
    }

    /// Returns the requested output identifier.
    #[must_use]
    pub const fn id(&self) -> &OutputId {
        &self.id
    }

    /// Returns the requested output configuration.
    #[must_use]
    pub const fn config(&self) -> OutputConfig {
        self.config
    }
}

/// Request to bind a session and workspace to an output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindSession {
    session: SessionId,
    output: OutputId,
    workspace: WorkspaceId,
}

impl BindSession {
    /// Creates a session binding request.
    #[must_use]
    pub const fn new(session: SessionId, output: OutputId, workspace: WorkspaceId) -> Self {
        Self {
            session,
            output,
            workspace,
        }
    }

    /// Returns the session identifier.
    #[must_use]
    pub const fn session(&self) -> &SessionId {
        &self.session
    }

    /// Returns the output identifier.
    #[must_use]
    pub const fn output(&self) -> &OutputId {
        &self.output
    }

    /// Returns the workspace identifier.
    #[must_use]
    pub const fn workspace(&self) -> &WorkspaceId {
        &self.workspace
    }
}

/// Status for a session binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingStatus {
    session: SessionId,
    output: OutputId,
    workspace: WorkspaceId,
}

impl BindingStatus {
    /// Creates a binding status value.
    #[must_use]
    pub const fn new(session: SessionId, output: OutputId, workspace: WorkspaceId) -> Self {
        Self {
            session,
            output,
            workspace,
        }
    }

    /// Returns the session identifier.
    #[must_use]
    pub const fn session(&self) -> &SessionId {
        &self.session
    }

    /// Returns the output identifier.
    #[must_use]
    pub const fn output(&self) -> &OutputId {
        &self.output
    }

    /// Returns the workspace identifier.
    #[must_use]
    pub const fn workspace(&self) -> &WorkspaceId {
        &self.workspace
    }
}

/// Desired compositor state used by reconciliation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReconcileState {
    outputs: Vec<CreateOutput>,
    bindings: Vec<BindSession>,
}

impl ReconcileState {
    /// Creates an empty desired state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            outputs: Vec::new(),
            bindings: Vec::new(),
        }
    }

    /// Adds an output to the desired state.
    pub fn add_output(&mut self, output: CreateOutput) {
        self.outputs.push(output);
    }

    /// Adds a binding to the desired state.
    pub fn add_binding(&mut self, binding: BindSession) {
        self.bindings.push(binding);
    }

    /// Returns desired outputs.
    #[must_use]
    pub fn outputs(&self) -> &[CreateOutput] {
        &self.outputs
    }

    /// Returns desired bindings.
    #[must_use]
    pub fn bindings(&self) -> &[BindSession] {
        &self.bindings
    }
}

/// Summary of reconciliation work performed by an adapter.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReconcileReport {
    /// Number of outputs created.
    pub created: u32,
    /// Number of outputs reconfigured.
    pub configured: u32,
    /// Number of outputs parked.
    pub parked: u32,
    /// Number of outputs removed.
    pub removed: u32,
    /// Number of session bindings applied.
    pub bound: u32,
}
