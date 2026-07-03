#![doc = "Platform-neutral captured frame metadata contract for madobe."]
#![forbid(unsafe_code)]
#![allow(
    clippy::module_name_repetitions,
    reason = "The public contract uses explicit capture and DMA-BUF domain names."
)]

/// Pixel size of the captured image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureSize {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl CaptureSize {
    /// Creates a capture size value.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// DRM four-character format code rendered as a stable evidence token.
///
/// This contract stores the token exactly as metadata. It does not interpret
/// the value as a platform API constant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DmaBufFormat {
    /// Human-readable fourcc token such as `XR24`.
    pub fourcc: String,
    /// Optional numeric DRM fourcc value when a capture backend records it.
    pub drm_format_code: Option<u32>,
}

impl DmaBufFormat {
    /// Creates a DMA-BUF format descriptor.
    #[must_use]
    pub const fn new(fourcc: String, drm_format_code: Option<u32>) -> Self {
        Self {
            fourcc,
            drm_format_code,
        }
    }
}

/// DMA-BUF modifier evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmaBufModifier {
    /// No modifier metadata was captured.
    Unknown,
    /// Explicit linear layout.
    Linear,
    /// Explicit numeric DRM modifier value.
    Drm(u64),
}

impl DmaBufModifier {
    /// Returns the stable lower-case evidence token for this modifier kind.
    #[must_use]
    pub const fn evidence_kind(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Linear => "linear",
            Self::Drm(_) => "drm",
        }
    }
}

/// One opaque DMA-BUF file descriptor number observed by a capture backend.
///
/// The number is evidence only. This type does not own, duplicate, close, or
/// otherwise operate on the descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DmaBufFileDescriptor {
    /// Backend-local file descriptor number.
    pub value: i32,
}

impl DmaBufFileDescriptor {
    /// Creates an opaque file descriptor evidence value.
    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self { value }
    }
}

/// Byte layout for one plane in a DMA-BUF-backed frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DmaBufPlane {
    /// Index into [`DmaBufFrameDescriptor::file_descriptors`].
    pub fd_index: u32,
    /// Plane offset from the start of the DMA-BUF allocation.
    pub offset_bytes: u32,
    /// Distance in bytes between adjacent rows.
    pub stride_bytes: u32,
}

impl DmaBufPlane {
    /// Creates a DMA-BUF plane layout descriptor.
    #[must_use]
    pub const fn new(fd_index: u32, offset_bytes: u32, stride_bytes: u32) -> Self {
        Self {
            fd_index,
            offset_bytes,
            stride_bytes,
        }
    }
}

/// DMA-BUF storage metadata for a captured frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DmaBufFrameDescriptor {
    /// Pixel format metadata.
    pub format: DmaBufFormat,
    /// Format modifier metadata.
    pub modifier: DmaBufModifier,
    /// Opaque file descriptor numbers recorded by the backend.
    pub file_descriptors: Vec<DmaBufFileDescriptor>,
    /// Plane layout metadata.
    pub planes: Vec<DmaBufPlane>,
}

impl DmaBufFrameDescriptor {
    /// Creates a DMA-BUF frame descriptor.
    #[must_use]
    pub const fn new(
        format: DmaBufFormat,
        modifier: DmaBufModifier,
        file_descriptors: Vec<DmaBufFileDescriptor>,
        planes: Vec<DmaBufPlane>,
    ) -> Self {
        Self {
            format,
            modifier,
            file_descriptors,
            planes,
        }
    }
}

/// Rectangle in captured-frame pixel coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DamageRect {
    /// Left coordinate in pixels.
    pub x: u32,
    /// Top coordinate in pixels.
    pub y: u32,
    /// Rectangle width in pixels.
    pub width: u32,
    /// Rectangle height in pixels.
    pub height: u32,
}

impl DamageRect {
    /// Creates a damage rectangle.
    #[must_use]
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Damage evidence attached to a captured frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureDamage {
    /// No damage metadata was captured.
    Unknown,
    /// The whole frame should be treated as damaged.
    Full,
    /// A set of damaged rectangles in frame pixel coordinates.
    Rects(Vec<DamageRect>),
}

impl CaptureDamage {
    /// Returns the stable lower-case evidence token for this damage kind.
    #[must_use]
    pub const fn evidence_kind(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Full => "full",
            Self::Rects(_) => "rects",
        }
    }
}

/// Opaque sync file descriptor number observed by a capture backend.
///
/// The number is evidence only. This type does not own, wait on, duplicate, or
/// close the descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyncFileDescriptor {
    /// Backend-local file descriptor number.
    pub value: i32,
}

impl SyncFileDescriptor {
    /// Creates an opaque sync file descriptor evidence value.
    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self { value }
    }
}

/// Synchronization metadata for a captured frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureSync {
    /// No synchronization metadata was captured.
    Unknown,
    /// Producer and consumer use implicit synchronization.
    Implicit,
    /// Producer provided an explicit sync file descriptor.
    Explicit(SyncFileDescriptor),
}

impl CaptureSync {
    /// Returns the stable lower-case evidence token for this sync kind.
    #[must_use]
    pub const fn evidence_kind(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Implicit => "implicit",
            Self::Explicit(_) => "explicit",
        }
    }
}

/// Sender-local frame timing metadata in nanoseconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureTimestamps {
    /// Monotonic timestamp when capture started, in nanoseconds.
    pub capture_started_ns: u64,
    /// Monotonic timestamp when the frame became available, in nanoseconds.
    pub frame_available_ns: u64,
}

impl CaptureTimestamps {
    /// Creates capture timestamp metadata.
    #[must_use]
    pub const fn new(capture_started_ns: u64, frame_available_ns: u64) -> Self {
        Self {
            capture_started_ns,
            frame_available_ns,
        }
    }
}

/// Captured frame metadata suitable for evidence serialization.
///
/// This type describes metadata a capture backend may report. It does not
/// claim that any specific compositor, kernel, or graphics API is available.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedFrameMetadata {
    /// Monotonic sender-local frame identifier.
    pub frame_id: u64,
    /// Captured image size in pixels.
    pub size: CaptureSize,
    /// DMA-BUF storage metadata.
    pub dma_buf: DmaBufFrameDescriptor,
    /// Damage metadata.
    pub damage: CaptureDamage,
    /// Synchronization metadata.
    pub sync: CaptureSync,
    /// Sender-local timestamp metadata.
    pub timestamps: CaptureTimestamps,
}

impl CapturedFrameMetadata {
    /// Creates captured frame metadata.
    #[must_use]
    pub const fn new(
        frame_id: u64,
        size: CaptureSize,
        dma_buf: DmaBufFrameDescriptor,
        damage: CaptureDamage,
        sync: CaptureSync,
        timestamps: CaptureTimestamps,
    ) -> Self {
        Self {
            frame_id,
            size,
            dma_buf,
            damage,
            sync,
            timestamps,
        }
    }
}
