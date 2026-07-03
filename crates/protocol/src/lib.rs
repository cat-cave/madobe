#![doc = "Minimal protocol identity types for the madobe workspace."]
#![forbid(unsafe_code)]

/// Human-readable product name shared by M0 components.
pub const PRODUCT_NAME: &str = "madobe";

/// Initial protocol version used by the M0 hello proof.
pub const PROTOCOL_VERSION: u16 = 1;

/// A small typed hello payload shared by the host daemon and CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MadobeHello {
    product: &'static str,
    protocol_version: u16,
}

impl MadobeHello {
    /// Returns the canonical hello payload for this workspace build.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            product: PRODUCT_NAME,
            protocol_version: PROTOCOL_VERSION,
        }
    }

    /// Returns the product identifier.
    #[must_use]
    pub const fn product(self) -> &'static str {
        self.product
    }

    /// Returns the protocol version.
    #[must_use]
    pub const fn protocol_version(self) -> u16 {
        self.protocol_version
    }

    /// Renders a deterministic identity segment for command output.
    #[must_use]
    pub fn identity(self) -> String {
        format!(
            "{} {} protocol={}",
            self.product,
            env!("CARGO_PKG_VERSION"),
            self.protocol_version
        )
    }
}

impl Default for MadobeHello {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the canonical product identity string.
#[must_use]
pub const fn product_identity() -> &'static str {
    PRODUCT_NAME
}

#[cfg(test)]
mod tests {
    use super::{MadobeHello, PRODUCT_NAME, PROTOCOL_VERSION, product_identity};

    #[test]
    fn hello_carries_stable_identity() {
        let hello = MadobeHello::new();

        assert_eq!(hello.product(), PRODUCT_NAME);
        assert_eq!(hello.protocol_version(), PROTOCOL_VERSION);
        assert_eq!(hello.identity(), "madobe 0.1.0 protocol=1");
        assert_eq!(product_identity(), "madobe");
    }
}
