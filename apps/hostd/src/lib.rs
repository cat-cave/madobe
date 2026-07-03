#![doc = "Bootstrap status rendering for the madobe host daemon."]
#![forbid(unsafe_code)]

use madobe_protocol::MadobeHello;
use madobe_telemetry::bootstrap_event;

/// Returns the deterministic M0 host status line.
#[must_use]
pub fn status_line() -> String {
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
    use super::status_line;

    #[test]
    fn status_line_links_shared_crates() {
        assert_eq!(
            status_line(),
            "madobe 0.1.0 protocol=1 event=madobe.bootstrap ts=0 status=ok"
        );
    }
}
