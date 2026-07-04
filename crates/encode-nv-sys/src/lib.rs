#![doc = "Isolated Linux NVENC command boundary for madobe."]
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

/// Pixel format token accepted by ffmpeg rawvideo input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RawVideoPixelFormat {
    /// Packed RGB with one byte per component.
    Rgb24,
}

impl RawVideoPixelFormat {
    /// Returns the ffmpeg pixel format argument.
    #[must_use]
    pub const fn ffmpeg_name(self) -> &'static str {
        match self {
            Self::Rgb24 => "rgb24",
        }
    }
}

/// Low-level rawvideo input description for the ffmpeg process boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawVideoInput {
    /// Raw frame bytes path.
    pub path: PathBuf,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Frames per second.
    pub frame_rate: u32,
    /// Input pixel format.
    pub pixel_format: RawVideoPixelFormat,
}

impl RawVideoInput {
    /// Creates a rawvideo input descriptor.
    #[must_use]
    pub fn new(
        path: impl Into<PathBuf>,
        width: u32,
        height: u32,
        frame_rate: u32,
        pixel_format: RawVideoPixelFormat,
    ) -> Self {
        Self {
            path: path.into(),
            width,
            height,
            frame_rate,
            pixel_format,
        }
    }
}

/// AV1 NVENC preset selected by the safe encoder layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NvencPreset {
    /// Balanced realtime preset.
    P4,
}

impl NvencPreset {
    /// Returns the ffmpeg preset token.
    #[must_use]
    pub const fn ffmpeg_name(self) -> &'static str {
        match self {
            Self::P4 => "p4",
        }
    }
}

/// AV1 NVENC tuning selected by the safe encoder layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NvencTune {
    /// Low-latency tuning for interactive desktop streaming.
    Ll,
}

impl NvencTune {
    /// Returns the ffmpeg tuning token.
    #[must_use]
    pub const fn ffmpeg_name(self) -> &'static str {
        match self {
            Self::Ll => "ll",
        }
    }
}

/// Low-level AV1 NVENC command request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NvencAv1Request {
    /// Rawvideo input descriptor.
    pub input: RawVideoInput,
    /// Encoded IVF artifact path.
    pub output: PathBuf,
    /// Encoder preset.
    pub preset: NvencPreset,
    /// Encoder tuning.
    pub tune: NvencTune,
    /// GOP size in frames.
    pub gop_frames: u32,
    /// Constant quality value passed to ffmpeg `-cq`.
    pub constant_quality: u8,
}

impl NvencAv1Request {
    /// Creates an AV1 NVENC command request.
    #[must_use]
    pub fn new(
        input: RawVideoInput,
        output: impl Into<PathBuf>,
        preset: NvencPreset,
        tune: NvencTune,
        gop_frames: u32,
        constant_quality: u8,
    ) -> Self {
        Self {
            input,
            output: output.into(),
            preset,
            tune,
            gop_frames,
            constant_quality,
        }
    }
}

/// Fully expanded command invocation for audit logging and execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfmpegCommand {
    /// Executable path or name.
    pub program: PathBuf,
    /// Argument vector excluding `program`.
    pub args: Vec<String>,
}

impl FfmpegCommand {
    /// Creates a shell-escaped command line for evidence logs.
    #[must_use]
    pub fn display_command(&self) -> String {
        let mut parts = Vec::with_capacity(self.args.len() + 1);
        parts.push(shell_word(&self.program));
        parts.extend(self.args.iter().map(|arg| shell_arg(arg)));
        parts.join(" ")
    }
}

/// Builds ffmpeg command lines for the Linux NVENC system boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfmpegNvenc {
    program: PathBuf,
}

impl FfmpegNvenc {
    /// Creates a boundary that resolves `ffmpeg` from PATH at execution time.
    #[must_use]
    pub fn from_path() -> Self {
        Self {
            program: PathBuf::from("ffmpeg"),
        }
    }

    /// Creates a boundary with an explicit ffmpeg executable path.
    #[must_use]
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }

    /// Builds an AV1 NVENC command from low-level validated inputs.
    #[must_use]
    pub fn av1_command(&self, request: &NvencAv1Request) -> FfmpegCommand {
        let size = format!("{}x{}", request.input.width, request.input.height);
        let frame_rate = request.input.frame_rate.to_string();
        let gop_frames = request.gop_frames.to_string();
        let constant_quality = request.constant_quality.to_string();

        FfmpegCommand {
            program: self.program.clone(),
            args: vec![
                "-hide_banner".to_owned(),
                "-y".to_owned(),
                "-f".to_owned(),
                "rawvideo".to_owned(),
                "-pix_fmt".to_owned(),
                request.input.pixel_format.ffmpeg_name().to_owned(),
                "-s:v".to_owned(),
                size,
                "-r".to_owned(),
                frame_rate,
                "-i".to_owned(),
                request.input.path.display().to_string(),
                "-an".to_owned(),
                "-vf".to_owned(),
                "format=nv12".to_owned(),
                "-c:v".to_owned(),
                "av1_nvenc".to_owned(),
                "-preset".to_owned(),
                request.preset.ffmpeg_name().to_owned(),
                "-tune".to_owned(),
                request.tune.ffmpeg_name().to_owned(),
                "-rc".to_owned(),
                "vbr".to_owned(),
                "-cq".to_owned(),
                constant_quality,
                "-b:v".to_owned(),
                "0".to_owned(),
                "-g".to_owned(),
                gop_frames,
                "-frames:v".to_owned(),
                "1".to_owned(),
                "-f".to_owned(),
                "ivf".to_owned(),
                request.output.display().to_string(),
            ],
        }
    }
}

fn shell_word(path: &Path) -> String {
    shell_arg(&path.display().to_string())
}

fn shell_arg(value: &str) -> String {
    if !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | ':' | '='))
    {
        value.to_owned()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FfmpegNvenc, NvencAv1Request, NvencPreset, NvencTune, RawVideoInput, RawVideoPixelFormat,
        shell_arg,
    };

    #[test]
    fn av1_command_uses_nvenc_and_ivf_container() {
        let input = RawVideoInput::new("frame.rgb", 128, 72, 60, RawVideoPixelFormat::Rgb24);
        let request =
            NvencAv1Request::new(input, "sample.ivf", NvencPreset::P4, NvencTune::Ll, 60, 28);
        let command = FfmpegNvenc::from_path().av1_command(&request);

        assert_eq!(command.program.display().to_string(), "ffmpeg");
        assert!(
            command
                .args
                .windows(2)
                .any(|args| args == ["-c:v", "av1_nvenc"])
        );
        assert!(command.args.windows(2).any(|args| args == ["-f", "ivf"]));
        assert!(
            command
                .args
                .windows(2)
                .any(|args| args == ["-pix_fmt", "rgb24"])
        );
        assert!(
            command
                .args
                .windows(2)
                .any(|args| args == ["-vf", "format=nv12"])
        );
        assert!(command.display_command().contains("av1_nvenc"));
    }

    #[test]
    fn nvenc_preset_ffmpeg_name_matches_ffmpeg_token() {
        assert_eq!(NvencPreset::P4.ffmpeg_name(), "p4");
    }

    #[test]
    fn nvenc_tune_ffmpeg_name_matches_ffmpeg_token() {
        assert_eq!(NvencTune::Ll.ffmpeg_name(), "ll");
    }

    #[test]
    fn shell_arg_leaves_safe_ffmpeg_tokens_unquoted() {
        assert_eq!(shell_arg("av1_nvenc"), "av1_nvenc");
        assert_eq!(shell_arg("/tmp/frame-01.rgb"), "/tmp/frame-01.rgb");
        assert_eq!(shell_arg("format=nv12"), "format=nv12");
        assert_eq!(shell_arg("stream:0"), "stream:0");
    }

    #[test]
    fn shell_arg_quotes_empty_and_unsafe_values() {
        assert_eq!(shell_arg(""), "''");
        assert_eq!(shell_arg("two words"), "'two words'");
        assert_eq!(shell_arg("it's.raw"), "'it'\"'\"'s.raw'");
    }
}
