use std::path::PathBuf;

use crate::product_quic_smoke::{
    ProductQuicError, ProductQuicReceiveOptions, ProductQuicSendOptions,
    receive_checked_av1_sample, send_checked_av1_sample,
};
use crate::video_smoke::CHECKED_IN_AV1_SAMPLE;

const USAGE: &str = "\
usage: madobectl product-quic-smoke send --addr <host:port> --server-name <name> --server-cert-der <path> [--sample <path>] [--evidence-dir <dir>]
       madobectl product-quic-smoke receive --bind <host:port> [--cert-san <name-or-ip>]... [--evidence-dir <dir>]";

/// Runs the product QUIC AV1 smoke CLI helper.
///
/// # Errors
///
/// Returns an error when arguments are invalid or the sender/receiver run
/// fails.
pub fn run_cli<I, S>(args: I) -> Result<String, ProductQuicError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    match parse(args)? {
        Command::Send(options) => {
            let summary = send_checked_av1_sample(&options)?;
            Ok(format!(
                "product-quic-smoke send addr={} server_name={} payload_bytes={} sha256={} status=sent",
                summary.remote_addr,
                summary.server_name,
                summary.payload_bytes,
                summary.payload_sha256
            ))
        }
        Command::Receive(options) => {
            let summary = receive_checked_av1_sample(&options)?;
            Ok(format!(
                "product-quic-smoke receive bind={} peer={} payload_bytes={} sha256={} status=passed",
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
    Send(ProductQuicSendOptions),
    Receive(ProductQuicReceiveOptions),
}

fn parse<I, S>(args: I) -> Result<Command, ProductQuicError>
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
        _ => Err(ProductQuicError::usage(USAGE)),
    }
}

fn parse_send(args: &[String]) -> Result<Command, ProductQuicError> {
    let mut addr = None;
    let mut server_name = None;
    let mut server_cert_der = None;
    let mut sample_path = PathBuf::from(CHECKED_IN_AV1_SAMPLE);
    let mut artifact_dir = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--addr" => {
                addr = Some(value_after(args, index, "--addr")?);
                index += 2;
            }
            "--server-name" => {
                server_name = Some(value_after(args, index, "--server-name")?);
                index += 2;
            }
            "--server-cert-der" => {
                server_cert_der = Some(PathBuf::from(value_after(
                    args,
                    index,
                    "--server-cert-der",
                )?));
                index += 2;
            }
            "--sample" => {
                sample_path = PathBuf::from(value_after(args, index, "--sample")?);
                index += 2;
            }
            "--evidence-dir" => {
                artifact_dir = Some(PathBuf::from(value_after(args, index, "--evidence-dir")?));
                index += 2;
            }
            _ => return Err(ProductQuicError::usage(USAGE)),
        }
    }

    Ok(Command::Send(ProductQuicSendOptions {
        addr: required(addr, "--addr")?,
        server_name: required(server_name, "--server-name")?,
        server_cert_der: required(server_cert_der, "--server-cert-der")?,
        sample_path,
        artifact_dir,
    }))
}

fn parse_receive(args: &[String]) -> Result<Command, ProductQuicError> {
    let mut bind = None;
    let mut cert_subject_alt_names = Vec::new();
    let mut artifact_dir = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--bind" => {
                bind = Some(value_after(args, index, "--bind")?);
                index += 2;
            }
            "--cert-san" => {
                cert_subject_alt_names.push(value_after(args, index, "--cert-san")?);
                index += 2;
            }
            "--evidence-dir" => {
                artifact_dir = Some(PathBuf::from(value_after(args, index, "--evidence-dir")?));
                index += 2;
            }
            _ => return Err(ProductQuicError::usage(USAGE)),
        }
    }

    Ok(Command::Receive(ProductQuicReceiveOptions {
        bind: required(bind, "--bind")?,
        cert_subject_alt_names,
        artifact_dir,
    }))
}

fn value_after(
    args: &[String],
    index: usize,
    flag: &'static str,
) -> Result<String, ProductQuicError> {
    args.get(index + 1)
        .cloned()
        .ok_or_else(|| ProductQuicError::usage(format!("{flag} requires a value\n{USAGE}")))
}

fn required<T>(value: Option<T>, flag: &'static str) -> Result<T, ProductQuicError> {
    value.ok_or_else(|| ProductQuicError::usage(format!("{flag} is required\n{USAGE}")))
}
