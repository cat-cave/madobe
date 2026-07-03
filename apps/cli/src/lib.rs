#![doc = "Command rendering for the madobe CLI."]
#![forbid(unsafe_code)]

use madobe_protocol::MadobeHello;
use madobe_telemetry::bootstrap_event;

/// Runs a CLI command and returns process output.
///
/// # Errors
///
/// Returns a usage string when the command is unknown or has extra arguments.
pub fn run<I, S>(args: I) -> Result<String, &'static str>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut args = args.into_iter();
    let command = args.next();
    let extra = args.next();

    match (
        command.as_ref().map(AsRef::as_ref),
        extra.as_ref().map(AsRef::as_ref),
    ) {
        (Some("hello"), None) => Ok(hello_line()),
        _ => Err("usage: madobectl hello"),
    }
}

/// Returns the deterministic M0 hello line.
#[must_use]
pub fn hello_line() -> String {
    let hello = MadobeHello::new();
    let event = bootstrap_event();

    format!(
        "{} event={} ts={} status=ok",
        hello.identity(),
        event.name(),
        event.timestamp().as_unix_millis()
    )
}

#[cfg(test)]
mod tests {
    use super::{hello_line, run};

    #[test]
    fn hello_line_links_shared_crates() {
        assert_eq!(
            hello_line(),
            "madobe 0.1.0 protocol=1 event=madobe.bootstrap ts=0 status=ok"
        );
    }

    #[test]
    fn hello_command_renders_status() {
        let output = run(["hello"]);

        assert_eq!(
            output,
            Ok(String::from(
                "madobe 0.1.0 protocol=1 event=madobe.bootstrap ts=0 status=ok"
            ))
        );
    }

    #[test]
    fn unknown_command_returns_usage() {
        assert_eq!(run(["status"]), Err("usage: madobectl hello"));
    }
}
