#![doc = "Platform-neutral compositor adapter contract for madobe."]
#![forbid(unsafe_code)]
#![allow(
    clippy::module_name_repetitions,
    reason = "The public contract uses explicit domain names for typed adapter values."
)]

mod adapter;
mod config;
mod error;
mod ids;
mod status;

pub use adapter::CompositorAdapter;
pub use config::{ColorDepth, Dimensions, OutputConfig, OutputMode, Position, RefreshRate, Scale};
pub use error::{Capability, CompositorError, ConfigError, ErrorKind, Operation, Resource, Result};
pub use ids::{IdentifierError, OutputId, SessionId, WorkspaceId};
pub use status::{
    BindSession, BindingStatus, CreateOutput, OutputState, OutputStatus, ReconcileReport,
    ReconcileState,
};
