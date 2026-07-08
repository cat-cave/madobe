use std::error::Error;
use std::fmt::{self, Display};
use std::io;

/// Error returned by the product QUIC smoke harness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductQuicError {
    kind: ProductQuicErrorKind,
}

impl ProductQuicError {
    /// Returns the machine-readable error kind.
    #[must_use]
    pub const fn kind(&self) -> &ProductQuicErrorKind {
        &self.kind
    }

    pub(super) fn io(operation: &'static str, error: &io::Error) -> Self {
        Self {
            kind: ProductQuicErrorKind::Io {
                operation,
                message: error.to_string(),
            },
        }
    }

    pub(super) fn quic(operation: &'static str, error: impl Display) -> Self {
        Self {
            kind: ProductQuicErrorKind::Quic {
                operation,
                message: error.to_string(),
            },
        }
    }

    pub(super) fn runtime(error: &io::Error) -> Self {
        Self {
            kind: ProductQuicErrorKind::Runtime {
                message: error.to_string(),
            },
        }
    }

    pub(super) fn sample(error: &crate::video_smoke::SmokeError) -> Self {
        Self {
            kind: ProductQuicErrorKind::Sample {
                message: error.to_string(),
            },
        }
    }

    pub(super) fn tls(operation: &'static str, error: impl Display) -> Self {
        Self {
            kind: ProductQuicErrorKind::Tls {
                operation,
                message: error.to_string(),
            },
        }
    }

    pub(super) fn usage(message: impl Into<String>) -> Self {
        Self {
            kind: ProductQuicErrorKind::Usage {
                message: message.into(),
            },
        }
    }

    pub(super) fn validation(message: impl Into<String>) -> Self {
        Self {
            kind: ProductQuicErrorKind::Validation {
                message: message.into(),
            },
        }
    }

    pub(super) fn wire(message: impl Into<String>) -> Self {
        Self {
            kind: ProductQuicErrorKind::Wire {
                message: message.into(),
            },
        }
    }
}

impl Display for ProductQuicError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ProductQuicErrorKind::Io { operation, message } => {
                write!(formatter, "{operation}: {message}")
            }
            ProductQuicErrorKind::Quic { operation, message } => {
                write!(formatter, "quic {operation}: {message}")
            }
            ProductQuicErrorKind::Runtime { message } => write!(formatter, "runtime: {message}"),
            ProductQuicErrorKind::Sample { message } => write!(formatter, "sample: {message}"),
            ProductQuicErrorKind::Tls { operation, message } => {
                write!(formatter, "tls {operation}: {message}")
            }
            ProductQuicErrorKind::Usage { message } => formatter.write_str(message),
            ProductQuicErrorKind::Validation { message } => {
                write!(formatter, "validation: {message}")
            }
            ProductQuicErrorKind::Wire { message } => write!(formatter, "wire: {message}"),
        }
    }
}

impl Error for ProductQuicError {}

/// Machine-readable product QUIC smoke error variants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductQuicErrorKind {
    /// File, socket, or artifact I/O failed.
    Io {
        /// Operation being performed.
        operation: &'static str,
        /// Underlying I/O error text.
        message: String,
    },
    /// QUIC endpoint, handshake, stream, or connection operation failed.
    Quic {
        /// Operation being performed.
        operation: &'static str,
        /// Underlying QUIC error text.
        message: String,
    },
    /// Tokio runtime creation failed.
    Runtime {
        /// Underlying runtime error text.
        message: String,
    },
    /// Checked-in sample loading failed.
    Sample {
        /// Underlying sample error text.
        message: String,
    },
    /// TLS certificate or verifier setup failed.
    Tls {
        /// Operation being performed.
        operation: &'static str,
        /// Underlying TLS error text.
        message: String,
    },
    /// CLI or address usage was invalid.
    Usage {
        /// Usage failure text.
        message: String,
    },
    /// Receiver validation rejected metadata, payload bytes, or ack text.
    Validation {
        /// Validation failure text.
        message: String,
    },
    /// Product QUIC smoke wire encoding or decoding failed.
    Wire {
        /// Wire failure text.
        message: String,
    },
}
