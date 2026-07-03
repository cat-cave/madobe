use std::error::Error;
use std::fmt::{self, Display};

/// Result alias used by compositor adapter operations.
pub type Result<T> = std::result::Result<T, CompositorError>;

/// Adapter operation used for error context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// Create an output.
    Create,
    /// Configure an output.
    Configure,
    /// Park an output.
    Park,
    /// Remove an output.
    Remove,
    /// List outputs.
    List,
    /// Reconcile desired and actual compositor state.
    Reconcile,
    /// Bind a session and workspace to an output.
    Bind,
}

/// Capability reported by an adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    /// Creating compositor-owned remote outputs.
    CreateOutput,
    /// Configuring an existing output.
    ConfigureOutput,
    /// Parking an output for later reuse.
    ParkOutput,
    /// Removing an owned output.
    RemoveOutput,
    /// Listing compositor state.
    ListOutputs,
    /// Reconciling desired state after restart or drift.
    ReconcileState,
    /// Binding sessions and workspaces to outputs.
    BindSession,
    /// Setting the requested color depth.
    ColorDepth,
    /// Setting the requested refresh rate.
    RefreshRate,
    /// Setting the requested scale.
    Scale,
}

/// Resource type involved in an adapter failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Resource {
    /// Output resource.
    Output,
    /// Workspace resource.
    Workspace,
    /// Session resource.
    Session,
}

/// Typed adapter failure reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// The backend cannot satisfy a required capability.
    Unsupported {
        /// Missing capability.
        capability: Capability,
    },
    /// A required resource does not exist.
    NotFound {
        /// Missing resource type.
        resource: Resource,
        /// Missing resource identifier.
        id: String,
    },
    /// The requested resource already exists.
    AlreadyExists {
        /// Existing resource type.
        resource: Resource,
        /// Existing resource identifier.
        id: String,
    },
    /// The requested configuration is invalid.
    InvalidConfig {
        /// Human-readable validation reason.
        reason: String,
    },
    /// The compositor backend is unavailable.
    Unavailable {
        /// Backend availability failure reason.
        reason: String,
    },
    /// The compositor rejected the operation for permission reasons.
    PermissionDenied {
        /// Permission failure reason.
        reason: String,
    },
    /// The backend observed inconsistent compositor state.
    InvariantViolation {
        /// Invariant failure reason.
        reason: String,
    },
}

/// Adapter error with operation context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositorError {
    operation: Operation,
    kind: ErrorKind,
}

impl CompositorError {
    /// Creates an adapter error.
    #[must_use]
    pub const fn new(operation: Operation, kind: ErrorKind) -> Self {
        Self { operation, kind }
    }

    /// Returns the operation that failed.
    #[must_use]
    pub const fn operation(&self) -> Operation {
        self.operation
    }

    /// Returns the typed failure reason.
    #[must_use]
    pub const fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl Display for CompositorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "compositor {:?} failed: {:?}",
            self.operation, self.kind
        )
    }
}

impl Error for CompositorError {}

/// Output configuration construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigError {
    /// A mode dimension was zero.
    EmptyDimensions,
    /// A refresh rate was zero.
    MissingRefreshRate,
    /// A scale ratio included zero.
    InvalidScaleRatio,
}

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDimensions => formatter.write_str("output dimensions must be non-zero"),
            Self::MissingRefreshRate => formatter.write_str("refresh rate must be non-zero"),
            Self::InvalidScaleRatio => formatter.write_str("scale ratio must be non-zero"),
        }
    }
}

impl Error for ConfigError {}
