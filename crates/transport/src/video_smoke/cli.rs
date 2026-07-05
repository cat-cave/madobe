use std::path::PathBuf;

use crate::video_smoke::{
    CHECKED_IN_AV1_SAMPLE, SmokeError, receive_checked_av1_sample_on, send_checked_av1_sample,
};

const USAGE: &str = "\
usage: madobectl video-smoke send --addr <host:port> [--sample <path>] [--evidence-dir <dir>]
       madobectl video-smoke receive --bind <host:port> [--evidence-dir <dir>]";

/// Runs the dependency-free AV1 LAN smoke CLI helper.
///
/// # Errors
///
/// Returns an error when arguments are invalid or the sender/receiver run
/// fails.
pub fn run_cli<I, S>(args: I) -> Result<String, SmokeError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    match parse(args)? {
        Command::Send {
            addr,
            sample,
            evidence_dir,
        } => {
            let summary = send_checked_av1_sample(addr.as_str(), sample, evidence_dir.as_deref())?;
            Ok(format!(
                "video-smoke send addr={} payload_bytes={} sha256={} status=sent",
                summary.remote_addr, summary.payload_bytes, summary.metadata.payload_hash.value
            ))
        }
        Command::Receive { bind, evidence_dir } => {
            let summary = receive_checked_av1_sample_on(bind.as_str(), evidence_dir.as_deref())?;
            Ok(format!(
                "video-smoke receive bind={} peer={} payload_bytes={} sha256={} status=passed",
                summary.local_addr,
                summary.peer_addr,
                summary.payload_bytes,
                summary.payload_sha256
            ))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Command {
    Send {
        addr: String,
        sample: PathBuf,
        evidence_dir: Option<PathBuf>,
    },
    Receive {
        bind: String,
        evidence_dir: Option<PathBuf>,
    },
}

fn parse<I, S>(args: I) -> Result<Command, SmokeError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_owned())
        .collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("send") => parse_send(&args[1..]),
        Some("receive") => parse_receive(&args[1..]),
        _ => Err(SmokeError::usage(USAGE)),
    }
}

fn parse_send(args: &[String]) -> Result<Command, SmokeError> {
    let mut addr = None;
    let mut sample = PathBuf::from(CHECKED_IN_AV1_SAMPLE);
    let mut evidence_dir = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--addr" => {
                addr = Some(value_after(args, index, "--addr")?);
                index += 2;
            }
            "--sample" => {
                sample = PathBuf::from(value_after(args, index, "--sample")?);
                index += 2;
            }
            "--evidence-dir" => {
                evidence_dir = Some(PathBuf::from(value_after(args, index, "--evidence-dir")?));
                index += 2;
            }
            _ => return Err(SmokeError::usage(USAGE)),
        }
    }

    Ok(Command::Send {
        addr: required(addr, "--addr")?,
        sample,
        evidence_dir,
    })
}

fn parse_receive(args: &[String]) -> Result<Command, SmokeError> {
    let mut bind = None;
    let mut evidence_dir = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--bind" => {
                bind = Some(value_after(args, index, "--bind")?);
                index += 2;
            }
            "--evidence-dir" => {
                evidence_dir = Some(PathBuf::from(value_after(args, index, "--evidence-dir")?));
                index += 2;
            }
            _ => return Err(SmokeError::usage(USAGE)),
        }
    }

    Ok(Command::Receive {
        bind: required(bind, "--bind")?,
        evidence_dir,
    })
}

fn value_after(args: &[String], index: usize, flag: &'static str) -> Result<String, SmokeError> {
    let value = args
        .get(index + 1)
        .ok_or_else(|| SmokeError::usage(format!("{flag} requires a value\n{USAGE}")))?;

    if value.starts_with("--") {
        return Err(SmokeError::usage(format!(
            "{flag} requires a value, got flag {value}\n{USAGE}"
        )));
    }

    Ok(value.clone())
}

fn required(value: Option<String>, flag: &'static str) -> Result<String, SmokeError> {
    value.ok_or_else(|| SmokeError::usage(format!("{flag} is required\n{USAGE}")))
}

#[cfg(test)]
mod tests {
    use super::run_cli;
    use crate::video_smoke::SmokeErrorKind;

    fn assert_usage<const N: usize>(args: [&str; N], expected_message: &str) {
        let error = match run_cli(args) {
            Ok(output) => panic!("expected usage error, got {output}"),
            Err(error) => error,
        };

        assert!(matches!(error.kind(), SmokeErrorKind::Usage { .. }));
        assert!(error.to_string().contains(expected_message));
        assert!(error.to_string().contains("madobectl video-smoke send"));
        assert!(error.to_string().contains("madobectl video-smoke receive"));
    }

    #[test]
    fn unknown_video_smoke_command_returns_usage_without_running_network() {
        assert_usage(["status"], "madobectl video-smoke send");
    }

    #[test]
    fn missing_required_video_smoke_flag_returns_usage_without_running_network() {
        assert_usage(["send"], "--addr is required");
    }

    #[test]
    fn missing_value_for_video_smoke_flag_returns_usage_without_running_network() {
        assert_usage(["send", "--addr"], "--addr requires a value");
        assert_usage(
            ["send", "--addr", "127.0.0.1:0", "--sample"],
            "--sample requires a value",
        );
        assert_usage(
            ["send", "--addr", "127.0.0.1:0", "--evidence-dir"],
            "--evidence-dir requires a value",
        );
        assert_usage(["receive", "--bind"], "--bind requires a value");
        assert_usage(
            ["receive", "--bind", "127.0.0.1:0", "--evidence-dir"],
            "--evidence-dir requires a value",
        );
    }

    #[test]
    fn flag_like_video_smoke_flag_value_returns_usage_without_running_network() {
        assert_usage(
            ["send", "--addr", "--sample"],
            "--addr requires a value, got flag --sample",
        );
        assert_usage(
            [
                "send",
                "--addr",
                "127.0.0.1:0",
                "--sample",
                "--evidence-dir",
            ],
            "--sample requires a value, got flag --evidence-dir",
        );
        assert_usage(
            [
                "send",
                "--addr",
                "127.0.0.1:0",
                "--evidence-dir",
                "--sample",
            ],
            "--evidence-dir requires a value, got flag --sample",
        );
        assert_usage(
            ["receive", "--bind", "--evidence-dir"],
            "--bind requires a value, got flag --evidence-dir",
        );
        assert_usage(
            [
                "receive",
                "--bind",
                "127.0.0.1:0",
                "--evidence-dir",
                "--bind",
            ],
            "--evidence-dir requires a value, got flag --bind",
        );
    }
}
