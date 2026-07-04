#![doc = "Command rendering for the madobe CLI."]
#![forbid(unsafe_code)]

use hostd::{DisplayAction, HostControlError};
use madobe_compositor::{CompositorAdapter, IdentifierError, OutputId, SessionId, WorkspaceId};
use madobe_protocol::MadobeHello;
use madobe_telemetry::bootstrap_event;
use std::error::Error;
use std::fmt::{self, Display};

const USAGE: &str = "\
usage: madobectl hello
       madobectl display status
       madobectl display create [--id <madobe-output-id>]
       madobectl display park [--id <madobe-output-id>]
       madobectl display remove [--id <madobe-output-id>]
       madobectl display bind --id <madobe-output-id> --session <session-id> --workspace <workspace-id>
       madobectl display smoke [--id <madobe-output-id>]
       madobectl video-smoke send --addr <host:port> [--sample <path>] [--evidence-dir <dir>]
       madobectl video-smoke receive --bind <host:port> [--evidence-dir <dir>]";

/// Runs an adapter-independent CLI command and returns process output.
///
/// # Errors
///
/// Returns a CLI error when the command is unknown, arguments are invalid, or the command requires
/// a compositor adapter.
pub fn run<I, S>(args: I) -> Result<String, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    match parse(args)? {
        Command::Hello => Ok(hello_line()),
        Command::Display(_) => Err(CliError::Usage),
    }
}

/// Runs a CLI command with an injected compositor adapter.
///
/// # Errors
///
/// Returns a CLI error when the command is unknown, arguments are invalid, or the compositor
/// operation fails.
pub fn run_with_adapter<I, S>(
    args: I,
    adapter: &mut impl CompositorAdapter,
) -> Result<String, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    match parse(args)? {
        Command::Hello => Ok(hello_line()),
        Command::Display(action) => hostd::run_display_action(adapter, action).map_err(Into::into),
    }
}

/// Returns whether a parsed command needs a compositor adapter.
///
/// # Errors
///
/// Returns a CLI error when the command is unknown or arguments are invalid.
pub fn requires_compositor_adapter<I, S>(args: I) -> Result<bool, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    Ok(matches!(parse(args)?, Command::Display(_)))
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

/// CLI command failure.
#[derive(Debug)]
pub enum CliError {
    /// The command shape is invalid.
    Usage,
    /// An output id failed validation.
    Identifier(IdentifierError),
    /// hostd rejected the display operation.
    Host(HostControlError),
}

impl Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str(USAGE),
            Self::Identifier(error) => write!(formatter, "invalid output id: {error}"),
            Self::Host(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for CliError {}

impl From<IdentifierError> for CliError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<HostControlError> for CliError {
    fn from(error: HostControlError) -> Self {
        Self::Host(error)
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Hello,
    Display(DisplayAction),
}

fn parse<I, S>(args: I) -> Result<Command, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_owned())
        .collect::<Vec<_>>();

    match args.as_slice() {
        [command] if command == "hello" => Ok(Command::Hello),
        [command, subcommand] if command == "display" && subcommand == "status" => {
            Ok(Command::Display(DisplayAction::Status))
        }
        [command, subcommand] if command == "display" => {
            parse_display_action(subcommand, hostd::default_smoke_output_id()?)
        }
        [command, subcommand, flag, value]
            if command == "display" && flag == "--id" && is_lifecycle_subcommand(subcommand) =>
        {
            parse_display_action(subcommand, OutputId::new(value)?)
        }
        [
            command,
            subcommand,
            id_flag,
            id,
            session_flag,
            session,
            workspace_flag,
            workspace,
        ] if command == "display"
            && subcommand == "bind"
            && id_flag == "--id"
            && session_flag == "--session"
            && workspace_flag == "--workspace" =>
        {
            Ok(Command::Display(DisplayAction::Bind {
                id: OutputId::new(id)?,
                session: SessionId::new(session)?,
                workspace: WorkspaceId::new(workspace)?,
            }))
        }
        _ => Err(CliError::Usage),
    }
}

fn parse_display_action(subcommand: &str, id: OutputId) -> Result<Command, CliError> {
    match subcommand {
        "create" => Ok(Command::Display(DisplayAction::Create { id })),
        "park" => Ok(Command::Display(DisplayAction::Park { id })),
        "remove" => Ok(Command::Display(DisplayAction::Remove { id })),
        "smoke" => Ok(Command::Display(DisplayAction::Smoke { id })),
        _ => Err(CliError::Usage),
    }
}

fn is_lifecycle_subcommand(subcommand: &str) -> bool {
    matches!(subcommand, "create" | "park" | "remove" | "smoke")
}

#[cfg(test)]
mod tests {
    use super::{CliError, hello_line, requires_compositor_adapter, run, run_with_adapter};
    use madobe_compositor::{
        BindSession, BindingStatus, CompositorAdapter, CompositorError, CreateOutput, ErrorKind,
        Operation, OutputConfig, OutputId, OutputState, OutputStatus, ReconcileReport,
        ReconcileState, Resource, Result,
    };
    use std::collections::BTreeMap;

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
            must(output),
            String::from("madobe 0.1.0 protocol=1 event=madobe.bootstrap ts=0 status=ok")
        );
    }

    #[test]
    fn display_status_command_renders_adapter_output() {
        let mut adapter = MockAdapter::with_output("madobe-cli-output");
        let output = run_with_adapter(["display", "status"], &mut adapter);

        assert_eq!(
            must(output),
            "display status count=1\noutput id=madobe-cli-output state=ready size=1280x720 refresh_millihertz=60000 scale=1/1 position=50000x50000 color_depth=8 workspace=-"
        );
    }

    #[test]
    fn display_command_without_adapter_returns_usage() {
        assert!(matches!(run(["display", "status"]), Err(CliError::Usage)));
    }

    #[test]
    fn adapter_requirement_is_detected_after_parse() {
        assert!(!must(requires_compositor_adapter(["hello"])));
        assert!(must(requires_compositor_adapter(["display", "status"])));
        assert!(matches!(
            requires_compositor_adapter(["unknown"]),
            Err(CliError::Usage)
        ));
    }

    #[test]
    fn display_lifecycle_command_accepts_explicit_output_id() {
        let mut adapter = MockAdapter::default();
        let output = run_with_adapter(
            ["display", "smoke", "--id", "madobe-explicit-smoke"],
            &mut adapter,
        );

        assert_eq!(
            must(output),
            "display smoke id=madobe-explicit-smoke create=ready park=parked remove=removed"
        );
        assert_eq!(
            adapter.calls,
            vec![
                "create:madobe-explicit-smoke",
                "park:madobe-explicit-smoke",
                "remove:madobe-explicit-smoke",
            ]
        );
    }

    #[test]
    fn display_lifecycle_command_has_deterministic_default_output_id() {
        let mut adapter = MockAdapter::default();
        let output = run_with_adapter(["display", "create"], &mut adapter);

        assert_eq!(
            must(output),
            String::from(
                "display create id=madobe-cli-smoke state=ready size=1280x720 refresh_millihertz=60000 scale=1/1 position=50000x50000 color_depth=8 workspace=-"
            )
        );
    }

    #[test]
    fn display_bind_command_accepts_session_and_workspace_ids() {
        let mut adapter = MockAdapter::with_output("madobe-cli-output");
        let output = run_with_adapter(
            [
                "display",
                "bind",
                "--id",
                "madobe-cli-output",
                "--session",
                "madobe-cli-session",
                "--workspace",
                "madobe-cli-workspace",
            ],
            &mut adapter,
        );

        assert_eq!(
            must(output),
            "display bind id=madobe-cli-output session=madobe-cli-session workspace=madobe-cli-workspace status=bound"
        );
        assert_eq!(
            adapter.calls,
            vec!["bind:madobe-cli-session:madobe-cli-output:madobe-cli-workspace"]
        );
    }

    #[test]
    fn unknown_command_returns_usage() {
        assert!(matches!(run(["status"]), Err(CliError::Usage)));
        let error = match run(["status"]) {
            Ok(output) => panic!("expected usage error, got {output}"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("madobectl display status"));
    }

    #[test]
    fn top_level_usage_lists_video_smoke_entry_points() {
        let error = match run(["video-smoke"]) {
            Ok(output) => panic!("expected usage error, got {output}"),
            Err(error) => error,
        };
        let usage = error.to_string();

        assert!(usage.contains("madobectl video-smoke send --addr <host:port>"));
        assert!(usage.contains("madobectl video-smoke receive --bind <host:port>"));
    }

    #[derive(Default)]
    struct MockAdapter {
        outputs: BTreeMap<OutputId, OutputStatus>,
        calls: Vec<String>,
    }

    impl MockAdapter {
        fn with_output(value: &str) -> Self {
            let id = id(value);
            let status = OutputStatus::new(
                id.clone(),
                must(hostd::default_display_config()),
                OutputState::Ready,
                None,
            );

            Self {
                outputs: BTreeMap::from([(id, status)]),
                calls: Vec::new(),
            }
        }
    }

    impl CompositorAdapter for MockAdapter {
        fn create_output(&mut self, request: CreateOutput) -> Result<OutputStatus> {
            self.calls.push(format!("create:{}", request.id()));
            let status = OutputStatus::new(
                request.id().clone(),
                request.config(),
                OutputState::Ready,
                None,
            );
            self.outputs.insert(request.id().clone(), status.clone());

            Ok(status)
        }

        fn configure_output(
            &mut self,
            id: &OutputId,
            config: OutputConfig,
        ) -> Result<OutputStatus> {
            self.calls.push(format!("configure:{id}"));
            let status = self.outputs.get_mut(id).ok_or_else(|| missing(id))?;
            let updated = OutputStatus::new(
                id.clone(),
                config,
                status.state(),
                status.workspace().cloned(),
            );
            *status = updated.clone();

            Ok(updated)
        }

        fn park_output(&mut self, id: &OutputId) -> Result<OutputStatus> {
            self.calls.push(format!("park:{id}"));
            let status = self.outputs.get_mut(id).ok_or_else(|| missing(id))?;
            let updated = OutputStatus::new(id.clone(), status.config(), OutputState::Parked, None);
            *status = updated.clone();

            Ok(updated)
        }

        fn remove_output(&mut self, id: &OutputId) -> Result<()> {
            self.calls.push(format!("remove:{id}"));
            self.outputs.remove(id).ok_or_else(|| missing(id))?;

            Ok(())
        }

        fn list_outputs(&self) -> Result<Vec<OutputStatus>> {
            Ok(self.outputs.values().cloned().collect())
        }

        fn reconcile(&mut self, _desired: &ReconcileState) -> Result<ReconcileReport> {
            Err(CompositorError::new(
                Operation::Reconcile,
                ErrorKind::InvariantViolation {
                    reason: "mock reconcile not used".to_owned(),
                },
            ))
        }

        fn bind_session(&mut self, request: BindSession) -> Result<BindingStatus> {
            self.calls.push(format!(
                "bind:{}:{}:{}",
                request.session(),
                request.output(),
                request.workspace()
            ));
            let status = self
                .outputs
                .get_mut(request.output())
                .ok_or_else(|| missing(request.output()))?;
            *status = OutputStatus::new(
                request.output().clone(),
                status.config(),
                OutputState::Bound,
                Some(request.workspace().clone()),
            );

            Ok(BindingStatus::new(
                request.session().clone(),
                request.output().clone(),
                request.workspace().clone(),
            ))
        }
    }

    fn id(value: &str) -> OutputId {
        must(OutputId::new(value))
    }

    fn missing(id: &OutputId) -> CompositorError {
        CompositorError::new(
            Operation::List,
            ErrorKind::NotFound {
                resource: Resource::Output,
                id: id.to_string(),
            },
        )
    }

    fn must<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        }
    }
}
