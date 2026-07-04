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
    if !value.is_finite() || value < 0.5 || value > f64::from(u32::MAX) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f64_to_nonzero_u32_rejects_non_positive_and_out_of_range_values() {
        for value in [f64::NEG_INFINITY, -1.0, -0.0, 0.0, f64::INFINITY, f64::NAN] {
            let error = must_err(
                f64_to_nonzero_u32(value, "scale", Operation::List),
                "invalid monitor field",
            );

            assert_eq!(error.operation(), Operation::List);
            assert!(
                matches!(error.kind(), ErrorKind::InvalidConfig { reason } if reason == "Hyprland monitor scale is out of range")
            );
        }

        assert!(f64_to_nonzero_u32(0.49, "scale", Operation::List).is_err());
        assert!(f64_to_nonzero_u32(f64::from(u32::MAX) + 1.0, "scale", Operation::List).is_err());
    }

    #[test]
    fn f64_to_nonzero_u32_accepts_rounded_u32_boundaries() {
        assert_eq!(
            must(
                f64_to_nonzero_u32(0.5, "refresh rate", Operation::List),
                "first rounded nonzero value",
            ),
            1
        );
        assert_eq!(
            must(
                f64_to_nonzero_u32(1.49, "refresh rate", Operation::List),
                "rounded value",
            ),
            1
        );
        assert_eq!(
            must(
                f64_to_nonzero_u32(1.5, "refresh rate", Operation::List),
                "rounded value",
            ),
            2
        );
        assert_eq!(
            must(
                f64_to_nonzero_u32(f64::from(u32::MAX), "refresh rate", Operation::List),
                "u32 max boundary",
            ),
            u32::MAX
        );
    }

    #[test]
    fn format_scale_formats_integer_and_fractional_scales() {
        assert_eq!(format_scale(Scale::one()), "1");
        assert_eq!(format_scale(scale(2, 1)), "2");
        assert_eq!(format_scale(scale(4, 2)), "2");
        assert_eq!(format_scale(scale(3, 2)), "1.5");
        assert_eq!(format_scale(scale(5, 4)), "1.25");
        assert_eq!(format_scale(scale(4, 3)), "1.333");
    }

    #[test]
    fn monitor_rule_formats_scale_via_config() {
        let config = OutputConfig::new(
            OutputMode::new(
                must(Dimensions::new(2560, 1440), "dimensions"),
                must(RefreshRate::from_millihertz(59_940), "refresh rate"),
            ),
            scale(4, 3),
            Position { x: -10, y: 20 },
            ColorDepth::Eight,
        );

        assert_eq!(
            monitor_rule(&must(OutputId::new("madobe-output"), "output id"), config),
            "madobe-output,2560x1440@59.94,-10x20,1.333"
        );
    }

    #[test]
    fn lua_string_escapes_quotes_and_backslashes() {
        assert_eq!(
            lua_string("madobe\\output'quoted"),
            "'madobe\\\\output\\'quoted'"
        );
    }

    #[test]
    fn config_from_monitor_rounds_and_reduces_scale_thousandths() {
        let monitors = must(
            crate::parse_monitors(
                r#"[{
                "id": 7,
                "name": "madobe-output",
                "description": "madobe headless",
                "make": "madobe",
                "model": "remote",
                "width": 2560,
                "height": 1440,
                "physicalWidth": 0,
                "physicalHeight": 0,
                "refreshRate": 59.9396,
                "x": -40,
                "y": 25,
                "activeWorkspace": {
                    "id": 17,
                    "name": "madobe-workspace"
                },
                "scale": 1.25,
                "focused": false,
                "dpmsStatus": true,
                "disabled": false
            }]"#,
            ),
            "monitor json",
        );

        let Some(monitor) = monitors.first() else {
            panic!("monitor json should contain one monitor");
        };
        let config = must(config_from_monitor(monitor, Operation::List), "config");

        assert_eq!(config.mode().dimensions().width().get(), 2560);
        assert_eq!(config.mode().dimensions().height().get(), 1440);
        assert_eq!(config.mode().refresh_rate().as_millihertz().get(), 59_940);
        assert_eq!(config.scale(), scale(5, 4));
        assert_eq!(config.position(), Position { x: -40, y: 25 });
        assert_eq!(config.color_depth(), ColorDepth::Eight);
    }

    #[test]
    fn greatest_common_divisor_handles_zero_and_nontrivial_remainders() {
        assert_eq!(greatest_common_divisor(0, 1000), 1000);
        assert_eq!(greatest_common_divisor(1250, 1000), 250);
        assert_eq!(greatest_common_divisor(1333, 1000), 1);
        assert_eq!(greatest_common_divisor(48, 18), 6);
    }

    fn scale(numerator: u32, denominator: u32) -> Scale {
        must(Scale::new(numerator, denominator), "scale")
    }

    fn must<T, E: std::fmt::Debug>(result: std::result::Result<T, E>, context: &str) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{context}: {error:?}"),
        }
    }

    fn must_err<T: std::fmt::Debug, E>(result: std::result::Result<T, E>, context: &str) -> E {
        match result {
            Ok(value) => panic!("{context}: expected error, got {value:?}"),
            Err(error) => error,
        }
    }
}
