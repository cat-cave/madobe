#![doc = "Safe product-level video encode API for madobe."]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use madobe_encode_nv_sys::{
    FfmpegCommand, FfmpegNvenc, NvencAv1Request, NvencPreset, NvencTune, RawVideoInput,
    RawVideoPixelFormat,
};

/// Product-level desktop encode quality.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopEncodeQuality {
    /// Realtime screen-sharing quality for the Linux host.
    BalancedRealtime,
}

/// Safe AV1 encode settings for captured desktop frames.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopAv1EncodeSettings {
    /// Target frames per second.
    pub frame_rate: u32,
    /// Product quality tier.
    pub quality: DesktopEncodeQuality,
}

impl DesktopAv1EncodeSettings {
    /// Returns balanced realtime AV1 settings for desktop capture.
    #[must_use]
    pub const fn balanced_realtime(frame_rate: u32) -> Self {
        Self {
            frame_rate,
            quality: DesktopEncodeQuality::BalancedRealtime,
        }
    }
}

/// Pixel format for a captured CPU-visible sample frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapturedSamplePixelFormat {
    /// Packed RGB with one byte per component.
    Rgb24,
}

impl CapturedSamplePixelFormat {
    const fn raw_video_format(self) -> RawVideoPixelFormat {
        match self {
            Self::Rgb24 => RawVideoPixelFormat::Rgb24,
        }
    }
}

/// Captured sample frame bytes prepared for an encode proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedSampleFrame {
    /// Raw frame bytes path.
    pub path: PathBuf,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Sample pixel format.
    pub pixel_format: CapturedSamplePixelFormat,
}

impl CapturedSampleFrame {
    /// Creates a captured sample frame descriptor.
    #[must_use]
    pub fn new(
        path: impl Into<PathBuf>,
        width: u32,
        height: u32,
        pixel_format: CapturedSamplePixelFormat,
    ) -> Self {
        Self {
            path: path.into(),
            width,
            height,
            pixel_format,
        }
    }
}

/// Safe Linux AV1 encoder facade.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinuxNvencAv1Encoder {
    boundary: FfmpegNvenc,
}

impl LinuxNvencAv1Encoder {
    /// Creates an encoder facade backed by `ffmpeg` from PATH.
    #[must_use]
    pub fn from_path() -> Self {
        Self {
            boundary: FfmpegNvenc::from_path(),
        }
    }

    /// Builds the command used to encode one captured sample frame.
    #[must_use]
    pub fn sample_command(
        &self,
        settings: DesktopAv1EncodeSettings,
        frame: &CapturedSampleFrame,
        output: impl Into<PathBuf>,
    ) -> FfmpegCommand {
        let request = NvencAv1Request::new(
            RawVideoInput::new(
                frame.path.clone(),
                frame.width,
                frame.height,
                settings.frame_rate,
                frame.pixel_format.raw_video_format(),
            ),
            output,
            NvencPreset::P4,
            NvencTune::Ll,
            settings.frame_rate,
            constant_quality(settings.quality),
        );

        self.boundary.av1_command(&request)
    }
}

const fn constant_quality(quality: DesktopEncodeQuality) -> u8 {
    match quality {
        DesktopEncodeQuality::BalancedRealtime => 28,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CapturedSampleFrame, CapturedSamplePixelFormat, DesktopAv1EncodeSettings,
        LinuxNvencAv1Encoder,
    };

    #[test]
    fn safe_api_maps_product_settings_to_nvenc_boundary() {
        let settings = DesktopAv1EncodeSettings::balanced_realtime(60);
        let frame =
            CapturedSampleFrame::new("capture.rgb", 160, 90, CapturedSamplePixelFormat::Rgb24);
        let command = LinuxNvencAv1Encoder::from_path().sample_command(settings, &frame, "out.ivf");

        assert!(
            command
                .args
                .windows(2)
                .any(|args| args == ["-preset", "p4"])
        );
        assert!(command.args.windows(2).any(|args| args == ["-tune", "ll"]));
        assert!(command.args.windows(2).any(|args| args == ["-cq", "28"]));
        assert!(command.args.windows(2).any(|args| args == ["-g", "60"]));
    }
}
