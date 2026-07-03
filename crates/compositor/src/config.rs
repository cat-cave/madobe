use crate::ConfigError;
use std::num::NonZeroU32;

/// Output pixel dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Dimensions {
    width: NonZeroU32,
    height: NonZeroU32,
}

impl Dimensions {
    /// Creates pixel dimensions.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::EmptyDimensions`] when either side is zero.
    pub fn new(width: u32, height: u32) -> Result<Self, ConfigError> {
        let width = NonZeroU32::new(width).ok_or(ConfigError::EmptyDimensions)?;
        let height = NonZeroU32::new(height).ok_or(ConfigError::EmptyDimensions)?;

        Ok(Self { width, height })
    }

    /// Returns the width in pixels.
    #[must_use]
    pub const fn width(self) -> NonZeroU32 {
        self.width
    }

    /// Returns the height in pixels.
    #[must_use]
    pub const fn height(self) -> NonZeroU32 {
        self.height
    }
}

/// Refresh rate stored as millihertz.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RefreshRate(NonZeroU32);

impl RefreshRate {
    /// Creates a refresh rate from millihertz.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::MissingRefreshRate`] when the value is zero.
    pub fn from_millihertz(value: u32) -> Result<Self, ConfigError> {
        let value = NonZeroU32::new(value).ok_or(ConfigError::MissingRefreshRate)?;
        Ok(Self(value))
    }

    /// Returns the refresh rate in millihertz.
    #[must_use]
    pub const fn as_millihertz(self) -> NonZeroU32 {
        self.0
    }
}

/// Fractional output scale.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Scale {
    numerator: NonZeroU32,
    denominator: NonZeroU32,
}

impl Scale {
    /// Creates a scale ratio.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidScaleRatio`] when either side is zero.
    pub fn new(numerator: u32, denominator: u32) -> Result<Self, ConfigError> {
        let numerator = NonZeroU32::new(numerator).ok_or(ConfigError::InvalidScaleRatio)?;
        let denominator = NonZeroU32::new(denominator).ok_or(ConfigError::InvalidScaleRatio)?;

        Ok(Self {
            numerator,
            denominator,
        })
    }

    /// Returns a 1x scale value.
    #[must_use]
    pub const fn one() -> Self {
        Self {
            numerator: NonZeroU32::MIN,
            denominator: NonZeroU32::MIN,
        }
    }

    /// Returns the ratio numerator.
    #[must_use]
    pub const fn numerator(self) -> NonZeroU32 {
        self.numerator
    }

    /// Returns the ratio denominator.
    #[must_use]
    pub const fn denominator(self) -> NonZeroU32 {
        self.denominator
    }
}

/// Output position in compositor layout coordinates.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Position {
    /// Horizontal coordinate.
    pub x: i32,
    /// Vertical coordinate.
    pub y: i32,
}

/// Requested color depth for an output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorDepth {
    /// Eight bits per component.
    Eight,
    /// Ten bits per component.
    Ten,
    /// Twelve bits per component.
    Twelve,
}

/// Requested output mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputMode {
    dimensions: Dimensions,
    refresh_rate: RefreshRate,
}

impl OutputMode {
    /// Creates an output mode.
    #[must_use]
    pub const fn new(dimensions: Dimensions, refresh_rate: RefreshRate) -> Self {
        Self {
            dimensions,
            refresh_rate,
        }
    }

    /// Returns pixel dimensions.
    #[must_use]
    pub const fn dimensions(self) -> Dimensions {
        self.dimensions
    }

    /// Returns the refresh rate.
    #[must_use]
    pub const fn refresh_rate(self) -> RefreshRate {
        self.refresh_rate
    }
}

/// Desired output configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputConfig {
    mode: OutputMode,
    scale: Scale,
    position: Position,
    color_depth: ColorDepth,
}

impl OutputConfig {
    /// Creates an output configuration.
    #[must_use]
    pub const fn new(
        mode: OutputMode,
        scale: Scale,
        position: Position,
        color_depth: ColorDepth,
    ) -> Self {
        Self {
            mode,
            scale,
            position,
            color_depth,
        }
    }

    /// Returns the requested mode.
    #[must_use]
    pub const fn mode(self) -> OutputMode {
        self.mode
    }

    /// Returns the requested scale.
    #[must_use]
    pub const fn scale(self) -> Scale {
        self.scale
    }

    /// Returns the requested position.
    #[must_use]
    pub const fn position(self) -> Position {
        self.position
    }

    /// Returns the requested color depth.
    #[must_use]
    pub const fn color_depth(self) -> ColorDepth {
        self.color_depth
    }
}
