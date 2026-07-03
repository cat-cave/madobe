use madobe_compositor::{
    ColorDepth, CompositorError, Dimensions, ErrorKind, Operation, OutputConfig, OutputId,
    OutputMode, Position, RefreshRate, Scale,
};

/// Returns the Hyprland monitor rule for an output configuration.
#[must_use]
pub fn monitor_rule(id: &OutputId, config: OutputConfig) -> String {
    let mode = config.mode();
    let dimensions = mode.dimensions();
    let position = config.position();

    format!(
        "{},{}x{}@{},{}x{},{}",
        id,
        dimensions.width(),
        dimensions.height(),
        format_millihertz(mode.refresh_rate().as_millihertz().get()),
        position.x,
        position.y,
        format_scale(config.scale())
    )
}

pub(super) fn monitor_eval(id: &OutputId, config: OutputConfig) -> String {
    let mode = config.mode();
    let dimensions = mode.dimensions();
    let position = config.position();

    format!(
        "hl.monitor({{ output = {}, mode = '{}x{}@{}', position = '{}x{}', scale = {} }})",
        lua_string(id.as_str()),
        dimensions.width(),
        dimensions.height(),
        format_millihertz(mode.refresh_rate().as_millihertz().get()),
        position.x,
        position.y,
        format_scale(config.scale())
    )
}

pub(super) fn config_from_monitor(
    monitor: &crate::HyprlandMonitor,
    operation: Operation,
) -> Result<OutputConfig, CompositorError> {
    let millihertz =
        f64_to_nonzero_u32(monitor.refresh_rate() * 1000.0, "refresh rate", operation)?;
    let scale_thousandths = f64_to_nonzero_u32(monitor.scale() * 1000.0, "scale", operation)?;
    let divisor = greatest_common_divisor(scale_thousandths, 1000);

    let dimensions = Dimensions::new(monitor.width(), monitor.height())
        .map_err(|error| invalid_config(operation, error.to_string()))?;
    let refresh_rate = RefreshRate::from_millihertz(millihertz)
        .map_err(|error| invalid_config(operation, error.to_string()))?;
    let scale = Scale::new(scale_thousandths / divisor, 1000 / divisor)
        .map_err(|error| invalid_config(operation, error.to_string()))?;

    Ok(OutputConfig::new(
        OutputMode::new(dimensions, refresh_rate),
        scale,
        Position {
            x: monitor.position().x,
            y: monitor.position().y,
        },
        ColorDepth::Eight,
    ))
}

fn f64_to_nonzero_u32(
    value: f64,
    field: &str,
    operation: Operation,
) -> Result<u32, CompositorError> {
    if !value.is_finite() || value <= 0.0 || value > f64::from(u32::MAX) {
        return Err(invalid_config(
            operation,
            format!("Hyprland monitor {field} is out of range"),
        ));
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "value is finite, positive, and bounded to u32 before rounding"
    )]
    Ok(value.round() as u32)
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

fn lua_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('\'');
    for character in value.chars() {
        if character == '\\' || character == '\'' {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped.push('\'');

    escaped
}

const fn greatest_common_divisor(mut left: u32, mut right: u32) -> u32 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }

    left
}

const fn invalid_config(operation: Operation, reason: String) -> CompositorError {
    CompositorError::new(operation, ErrorKind::InvalidConfig { reason })
}
