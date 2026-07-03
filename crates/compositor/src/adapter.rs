use crate::{
    BindSession, BindingStatus, CompositorError, CreateOutput, OutputConfig, OutputId,
    OutputStatus, ReconcileReport, ReconcileState,
};

/// Platform-neutral compositor adapter contract.
pub trait CompositorAdapter {
    /// Creates a remote output owned by madobe.
    ///
    /// # Errors
    ///
    /// Returns [`CompositorError`] when the backend cannot create or adopt the
    /// requested output.
    fn create_output(&mut self, request: CreateOutput) -> Result<OutputStatus, CompositorError>;

    /// Applies output mode, scale, position, color depth, and refresh settings.
    ///
    /// # Errors
    ///
    /// Returns [`CompositorError`] when the output is missing or the backend
    /// cannot apply the requested configuration.
    fn configure_output(
        &mut self,
        id: &OutputId,
        config: OutputConfig,
    ) -> Result<OutputStatus, CompositorError>;

    /// Parks an owned output as a warm spare.
    ///
    /// # Errors
    ///
    /// Returns [`CompositorError`] when the output is missing or cannot be
    /// moved into the parked state.
    fn park_output(&mut self, id: &OutputId) -> Result<OutputStatus, CompositorError>;

    /// Removes an owned output.
    ///
    /// # Errors
    ///
    /// Returns [`CompositorError`] when the output is missing or cannot be
    /// removed.
    fn remove_output(&mut self, id: &OutputId) -> Result<(), CompositorError>;

    /// Lists outputs currently known to the adapter.
    ///
    /// # Errors
    ///
    /// Returns [`CompositorError`] when the backend cannot read compositor
    /// state.
    fn list_outputs(&self) -> Result<Vec<OutputStatus>, CompositorError>;

    /// Reconciles desired output and binding state with compositor state.
    ///
    /// # Errors
    ///
    /// Returns [`CompositorError`] when any required repair operation fails.
    fn reconcile(&mut self, desired: &ReconcileState) -> Result<ReconcileReport, CompositorError>;

    /// Binds a remote session and workspace to an output.
    ///
    /// # Errors
    ///
    /// Returns [`CompositorError`] when the output, workspace, or session cannot
    /// be bound.
    fn bind_session(&mut self, request: BindSession) -> Result<BindingStatus, CompositorError>;
}
