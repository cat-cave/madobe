#![doc = "Hyprland adapter lifecycle tests over a stateful command executor."]
#![forbid(unsafe_code)]

use madobe_compositor::{
    ColorDepth, CompositorAdapter, CreateOutput, Dimensions, ErrorKind, Operation, OutputConfig,
    OutputId, OutputMode, OutputState, Position, ReconcileReport, ReconcileState, RefreshRate,
    Scale,
};
use madobe_hyprland::{
    CommandExecutor, CommandOutput, HyprctlCommand, HyprlandAdapter, HyprlandError,
    HyprlandErrorKind, Result,
};
use std::cell::RefCell;
use std::collections::BTreeMap;

#[test]
fn creates_configures_parks_and_removes_named_headless_output() {
    let output = output("madobe-test-output");
    let initial = config(1920, 1080, 60_000, Position { x: 40_000, y: 0 });
    let updated = config(2560, 1440, 144_000, Position { x: 42_000, y: 0 });
    let executor = LifecycleExecutor::default();
    let mut adapter = must(HyprlandAdapter::new(executor));

    let created = must(adapter.create_output(CreateOutput::new(output.clone(), initial)));
    assert_eq!(created.id(), &output);
    assert_eq!(created.config(), initial);
    assert_eq!(created.state(), OutputState::Ready);

    assert_eq!(
        must(adapter.configure_output(&output, updated)).config(),
        updated
    );
    let listed = must(adapter.list_outputs());
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id(), &output);
    assert_eq!(listed[0].config(), updated);
    assert_eq!(
        must(adapter.park_output(&output)).state(),
        OutputState::Parked
    );

    must(adapter.remove_output(&output));
    must(adapter.remove_output(&output));
    assert!(must(adapter.list_outputs()).is_empty());

    assert_eq!(
        adapter.client().executor().calls(),
        vec![
            args(["-j", "monitors"]),
            args(["output", "create", "headless", "madobe-test-output"]),
            args(["eval", monitor_eval(&output, initial).as_str()]),
            args(["-j", "monitors"]),
            args(["-j", "monitors"]),
            args(["eval", monitor_eval(&output, updated).as_str()]),
            args(["-j", "monitors"]),
            args(["-j", "monitors"]),
            args(["-j", "monitors"]),
            args([
                "eval",
                monitor_eval(&output, adapter.parking_config()).as_str()
            ]),
            args(["-j", "monitors"]),
            args(["-j", "monitors"]),
            args(["output", "remove", "madobe-test-output"]),
            args(["-j", "monitors"]),
            args(["-j", "monitors"]),
        ]
    );
}

#[test]
fn reconciles_missing_and_stale_outputs_with_typed_report() {
    let desired_output = output("madobe-desired-output");
    let stale_output = output("madobe-stale-output");
    let desired_config = config(1920, 1080, 60_000, Position { x: 40_000, y: 0 });
    let stale_config = config(1280, 720, 60_000, Position { x: 43_000, y: 0 });
    let executor = LifecycleExecutor::with_outputs([(stale_output.clone(), stale_config)]);
    let mut adapter = must(HyprlandAdapter::new(executor));
    let mut desired = ReconcileState::new();
    desired.add_output(CreateOutput::new(desired_output.clone(), desired_config));

    assert_eq!(
        must(adapter.reconcile(&desired)),
        ReconcileReport {
            created: 1,
            configured: 0,
            parked: 1,
            removed: 0,
            bound: 0,
        }
    );

    let outputs = must(adapter.list_outputs());
    assert_eq!(outputs.len(), 2);
    assert!(
        outputs
            .iter()
            .any(|status| status.id() == &desired_output && status.state() == OutputState::Ready)
    );
    assert!(
        outputs
            .iter()
            .any(|status| status.id() == &stale_output && status.state() == OutputState::Parked)
    );
}

#[test]
fn rejects_unowned_outputs_and_unsupported_color_depth() {
    let executor = LifecycleExecutor::default();
    let mut adapter = must(HyprlandAdapter::new(executor));

    let error = match adapter.create_output(CreateOutput::new(
        output("other-output"),
        config(1920, 1080, 60_000, Position { x: 0, y: 0 }),
    )) {
        Ok(status) => panic!("expected unowned output to fail, got {status:?}"),
        Err(error) => error,
    };

    assert_eq!(error.operation(), Operation::Create);
    assert!(
        matches!(error.kind(), ErrorKind::InvalidConfig { reason } if reason.contains("madobe-"))
    );

    let unsupported = OutputConfig::new(
        OutputMode::new(
            must(Dimensions::new(1920, 1080)),
            must(RefreshRate::from_millihertz(60_000)),
        ),
        Scale::one(),
        Position { x: 0, y: 0 },
        ColorDepth::Ten,
    );

    let error =
        match adapter.create_output(CreateOutput::new(output("madobe-hdr-output"), unsupported)) {
            Ok(status) => panic!("expected unsupported color depth to fail, got {status:?}"),
            Err(error) => error,
        };

    assert_eq!(error.operation(), Operation::Create);
    assert!(matches!(error.kind(), ErrorKind::Unsupported { .. }));
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OutputSnapshot {
    config: OutputConfig,
    workspace: String,
}

#[derive(Debug, Default)]
struct LifecycleExecutor {
    outputs: RefCell<BTreeMap<String, OutputSnapshot>>,
    calls: RefCell<Vec<Vec<String>>>,
}

impl LifecycleExecutor {
    fn with_outputs<const N: usize>(outputs: [(OutputId, OutputConfig); N]) -> Self {
        Self {
            outputs: RefCell::new(
                outputs
                    .into_iter()
                    .map(|(id, config)| {
                        (
                            id.to_string(),
                            OutputSnapshot {
                                config,
                                workspace: "madobe-workspace".to_owned(),
                            },
                        )
                    })
                    .collect(),
            ),
            calls: RefCell::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<Vec<String>> {
        self.calls.borrow().clone()
    }

    fn monitors_json(&self) -> String {
        let entries = self
            .outputs
            .borrow()
            .iter()
            .enumerate()
            .map(|(index, (name, snapshot))| monitor_json(index + 1, name, snapshot))
            .collect::<Vec<_>>();

        format!("[{}]", entries.join(","))
    }

    fn apply_monitor_rule(&self, rule: &str) -> std::result::Result<(), String> {
        let mut parts = rule.split(',');
        let name = next_part(&mut parts, "output name")?;
        let mode = next_part(&mut parts, "output mode")?;
        let position = next_part(&mut parts, "output position")?;
        let scale = next_part(&mut parts, "output scale")?;
        let (dimensions, refresh_rate) = split_once(mode, '@', "refresh rate")?;
        let (width, height) = split_once(dimensions, 'x', "dimensions")?;
        let (x, y) = split_once(position, 'x', "position")?;
        let config = OutputConfig::new(
            OutputMode::new(
                Dimensions::new(parse_u32(width, "width")?, parse_u32(height, "height")?)
                    .map_err(|error| error.to_string())?,
                RefreshRate::from_millihertz(parse_decimal_thousandths(refresh_rate, "refresh")?)
                    .map_err(|error| error.to_string())?,
            ),
            parse_scale(scale)?,
            Position {
                x: parse_i32(x, "x")?,
                y: parse_i32(y, "y")?,
            },
            ColorDepth::Eight,
        );

        self.outputs.borrow_mut().insert(
            name.to_owned(),
            OutputSnapshot {
                config,
                workspace: "madobe-workspace".to_owned(),
            },
        );

        Ok(())
    }

    fn apply_monitor_eval(&self, expression: &str) -> std::result::Result<(), String> {
        let output = extract_quoted(expression, "output")?;
        let mode = extract_quoted(expression, "mode")?;
        let position = extract_quoted(expression, "position")?;
        let scale = extract_scale(expression)?;

        self.apply_monitor_rule(&format!("{output},{mode},{position},{scale}"))
    }
}

impl CommandExecutor for LifecycleExecutor {
    fn execute(&self, command: &HyprctlCommand) -> Result<CommandOutput> {
        let args = command.args().to_vec();
        self.calls.borrow_mut().push(args.clone());

        if args_match(&args, &["-j", "monitors"]) {
            return Ok(CommandOutput::new(self.monitors_json(), ""));
        }
        if args.len() == 4 && args_match(&args[..3], &["output", "create", "headless"]) {
            self.outputs.borrow_mut().insert(
                args[3].clone(),
                OutputSnapshot {
                    config: config(1280, 720, 60_000, Position { x: 0, y: 0 }),
                    workspace: "madobe-workspace".to_owned(),
                },
            );
            return Ok(CommandOutput::new("ok", ""));
        }
        if args.len() == 3 && args_match(&args[..2], &["output", "remove"]) {
            self.outputs.borrow_mut().remove(&args[2]);
            return Ok(CommandOutput::new("ok", ""));
        }
        if args.len() == 2 && args_match(&args[..1], &["eval"]) {
            return self.apply_monitor_eval(&args[1]).map_or_else(
                |reason| Err(command_error(command, reason)),
                |()| Ok(CommandOutput::new("ok", "")),
            );
        }

        Err(command_error(command, "unexpected command".to_owned()))
    }
}

fn monitor_json(id: usize, name: &str, snapshot: &OutputSnapshot) -> String {
    let config = snapshot.config;
    let mode = config.mode();
    let dimensions = mode.dimensions();
    let position = config.position();

    format!(
        "{{\"id\":{},\"name\":\"{}\",\"description\":\"madobe headless\",\"make\":\"madobe\",\"model\":\"remote\",\"width\":{},\"height\":{},\"physicalWidth\":0,\"physicalHeight\":0,\"refreshRate\":{},\"x\":{},\"y\":{},\"activeWorkspace\":{{\"id\":{},\"name\":\"{}\"}},\"scale\":{},\"focused\":false,\"dpmsStatus\":true,\"disabled\":false}}",
        id,
        name,
        dimensions.width(),
        dimensions.height(),
        f64::from(mode.refresh_rate().as_millihertz().get()) / 1000.0,
        position.x,
        position.y,
        id + 100,
        snapshot.workspace,
        f64::from(config.scale().numerator().get()) / f64::from(config.scale().denominator().get())
    )
}

fn config(width: u32, height: u32, millihertz: u32, position: Position) -> OutputConfig {
    OutputConfig::new(
        OutputMode::new(
            must(Dimensions::new(width, height)),
            must(RefreshRate::from_millihertz(millihertz)),
        ),
        Scale::one(),
        position,
        ColorDepth::Eight,
    )
}

fn output(value: &str) -> OutputId {
    must(OutputId::new(value))
}

fn args<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(str::to_owned).collect()
}

fn args_match(args: &[String], expected: &[&str]) -> bool {
    args.len() == expected.len()
        && args
            .iter()
            .zip(expected.iter())
            .all(|(actual, expected)| actual == expected)
}

fn monitor_eval(id: &OutputId, config: OutputConfig) -> String {
    let mode = config.mode();
    let dimensions = mode.dimensions();
    let position = config.position();

    format!(
        "hl.monitor({{ output = '{}', mode = '{}x{}@{}', position = '{}x{}', scale = {} }})",
        id,
        dimensions.width(),
        dimensions.height(),
        format_millihertz(mode.refresh_rate().as_millihertz().get()),
        position.x,
        position.y,
        format_scale(config.scale())
    )
}

fn format_millihertz(value: u32) -> String {
    if value.is_multiple_of(1000) {
        return (value / 1000).to_string();
    }

    trim_decimal(format!("{:.3}", f64::from(value) / 1000.0))
}

fn format_scale(scale: Scale) -> String {
    let numerator = scale.numerator().get();
    let denominator = scale.denominator().get();

    if numerator.is_multiple_of(denominator) {
        return (numerator / denominator).to_string();
    }

    trim_decimal(format!(
        "{:.3}",
        f64::from(numerator) / f64::from(denominator)
    ))
}

fn trim_decimal(mut value: String) -> String {
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }

    value
}

fn extract_quoted(expression: &str, field: &str) -> std::result::Result<String, String> {
    let prefix = format!("{field} = '");
    let Some((_, rest)) = expression.split_once(&prefix) else {
        return Err(format!("missing {field}"));
    };
    let Some((value, _)) = rest.split_once('\'') else {
        return Err(format!("unterminated {field}"));
    };

    Ok(value.to_owned())
}

fn extract_scale(expression: &str) -> std::result::Result<String, String> {
    let Some((_, rest)) = expression.split_once("scale = ") else {
        return Err("missing scale".to_owned());
    };
    let Some((value, _)) = rest.split_once(" })") else {
        return Err("unterminated scale".to_owned());
    };

    Ok(value.to_owned())
}

fn next_part<'a>(
    parts: &mut impl Iterator<Item = &'a str>,
    label: &str,
) -> std::result::Result<&'a str, String> {
    parts.next().ok_or_else(|| format!("missing {label}"))
}

fn split_once<'a>(
    value: &'a str,
    delimiter: char,
    label: &str,
) -> std::result::Result<(&'a str, &'a str), String> {
    value
        .split_once(delimiter)
        .ok_or_else(|| format!("missing {label}"))
}

fn parse_u32(value: &str, field: &str) -> std::result::Result<u32, String> {
    value
        .parse()
        .map_err(|error| format!("invalid {field}: {error}"))
}

fn parse_i32(value: &str, field: &str) -> std::result::Result<i32, String> {
    value
        .parse()
        .map_err(|error| format!("invalid {field}: {error}"))
}

fn parse_scale(value: &str) -> std::result::Result<Scale, String> {
    let thousandths = parse_decimal_thousandths(value, "scale")?;
    let divisor = greatest_common_divisor(thousandths, 1000);

    Scale::new(thousandths / divisor, 1000 / divisor).map_err(|error| error.to_string())
}

fn parse_decimal_thousandths(value: &str, field: &str) -> std::result::Result<u32, String> {
    if value.starts_with('-') {
        return Err(format!(
            "invalid {field}: negative values are not supported"
        ));
    }

    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    let whole = parse_u32(whole, field)?;
    if fraction.len() > 3 {
        return Err(format!("invalid {field}: more than three decimal places"));
    }

    let mut padded_fraction = fraction.to_owned();
    while padded_fraction.len() < 3 {
        padded_fraction.push('0');
    }

    let fraction = parse_u32(&padded_fraction, field)?;
    whole
        .checked_mul(1000)
        .and_then(|value| value.checked_add(fraction))
        .ok_or_else(|| format!("invalid {field}: value is out of range"))
}

const fn greatest_common_divisor(mut left: u32, mut right: u32) -> u32 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }

    left
}

fn command_error(command: &HyprctlCommand, stderr: String) -> HyprlandError {
    HyprlandError::new(
        command.context(),
        HyprlandErrorKind::NonZeroExit {
            code: Some(1),
            stderr,
        },
    )
}

fn must<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error:?}"),
    }
}
