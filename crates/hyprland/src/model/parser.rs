use super::{HyprlandMonitor, HyprlandWorkspace, HyprlandWorkspaceRef};
use crate::{CommandContext, HyprlandError, HyprlandErrorKind, Result};

#[derive(Clone, Debug, PartialEq)]
enum JsonValue {
    Array(Vec<Self>),
    Object(Vec<(String, Self)>),
    String(String),
    Number(String),
    Bool(bool),
    Null,
}

pub(super) fn parse_monitors(
    payload: &str,
    context: CommandContext,
) -> Result<Vec<HyprlandMonitor>> {
    let value = parse_root(payload, context.clone())?;
    let JsonValue::Array(items) = value else {
        return Err(invalid_json(context, "expected monitor array"));
    };

    items
        .iter()
        .map(|item| monitor_from_value(item, &context))
        .collect()
}

pub(super) fn parse_workspaces(
    payload: &str,
    context: CommandContext,
) -> Result<Vec<HyprlandWorkspace>> {
    let value = parse_root(payload, context.clone())?;
    let JsonValue::Array(items) = value else {
        return Err(invalid_json(context, "expected workspace array"));
    };

    items
        .iter()
        .map(|item| workspace_from_value(item, &context))
        .collect()
}

pub(super) fn parse_active_workspace(
    payload: &str,
    context: CommandContext,
) -> Result<HyprlandWorkspace> {
    let value = parse_root(payload, context.clone())?;
    let workspace = workspace_from_value(&value, &context);
    drop(context);
    workspace
}

fn monitor_from_value(value: &JsonValue, context: &CommandContext) -> Result<HyprlandMonitor> {
    Ok(HyprlandMonitor {
        id: object_i64(value, "id", context)?,
        name: object_string(value, "name", context)?,
        description: object_string(value, "description", context)?,
        make: object_string(value, "make", context)?,
        model: object_string(value, "model", context)?,
        width: object_u32(value, "width", context)?,
        height: object_u32(value, "height", context)?,
        physical_width: object_u32(value, "physicalWidth", context)?,
        physical_height: object_u32(value, "physicalHeight", context)?,
        refresh_rate: object_f64(value, "refreshRate", context)?,
        x: object_i32(value, "x", context)?,
        y: object_i32(value, "y", context)?,
        active_workspace: workspace_ref_from_value(
            object_field(value, "activeWorkspace", context)?,
            context,
        )?,
        scale: object_f64(value, "scale", context)?,
        focused: object_bool(value, "focused", context)?,
        dpms_status: object_bool(value, "dpmsStatus", context)?,
        disabled: object_bool(value, "disabled", context)?,
    })
}

fn workspace_ref_from_value(
    value: &JsonValue,
    context: &CommandContext,
) -> Result<HyprlandWorkspaceRef> {
    Ok(HyprlandWorkspaceRef {
        id: object_i64(value, "id", context)?,
        name: object_string(value, "name", context)?,
    })
}

fn workspace_from_value(value: &JsonValue, context: &CommandContext) -> Result<HyprlandWorkspace> {
    Ok(HyprlandWorkspace {
        id: object_i64(value, "id", context)?,
        name: object_string(value, "name", context)?,
        monitor: object_string(value, "monitor", context)?,
        monitor_id: object_i64(value, "monitorID", context)?,
        windows: object_u32(value, "windows", context)?,
        hasfullscreen: object_bool(value, "hasfullscreen", context)?,
        lastwindow: object_string(value, "lastwindow", context)?,
        lastwindowtitle: object_string(value, "lastwindowtitle", context)?,
        ispersistent: object_bool(value, "ispersistent", context)?,
        tiled_layout: object_string(value, "tiledLayout", context)?,
    })
}

fn object_field<'a>(
    value: &'a JsonValue,
    key: &str,
    context: &CommandContext,
) -> Result<&'a JsonValue> {
    let JsonValue::Object(fields) = value else {
        return Err(invalid_json(context.clone(), "expected object"));
    };

    fields
        .iter()
        .find_map(|(field_key, field_value)| (field_key == key).then_some(field_value))
        .ok_or_else(|| invalid_json(context.clone(), &format!("missing field '{key}'")))
}

fn object_string(value: &JsonValue, key: &str, context: &CommandContext) -> Result<String> {
    match object_field(value, key, context)? {
        JsonValue::String(value) => Ok(value.clone()),
        _ => Err(invalid_json(
            context.clone(),
            &format!("field '{key}' must be a string"),
        )),
    }
}

fn object_bool(value: &JsonValue, key: &str, context: &CommandContext) -> Result<bool> {
    match object_field(value, key, context)? {
        JsonValue::Bool(value) => Ok(*value),
        _ => Err(invalid_json(
            context.clone(),
            &format!("field '{key}' must be a boolean"),
        )),
    }
}

fn object_i32(value: &JsonValue, key: &str, context: &CommandContext) -> Result<i32> {
    let parsed = object_i64(value, key, context)?;
    i32::try_from(parsed)
        .map_err(|error| invalid_json(context.clone(), &format!("field '{key}' {error}")))
}

fn object_i64(value: &JsonValue, key: &str, context: &CommandContext) -> Result<i64> {
    parse_number(object_field(value, key, context)?, key, context)
}

fn object_u32(value: &JsonValue, key: &str, context: &CommandContext) -> Result<u32> {
    let parsed = object_i64(value, key, context)?;
    u32::try_from(parsed)
        .map_err(|error| invalid_json(context.clone(), &format!("field '{key}' {error}")))
}

fn object_f64(value: &JsonValue, key: &str, context: &CommandContext) -> Result<f64> {
    parse_number(object_field(value, key, context)?, key, context)
}

fn parse_number<T>(value: &JsonValue, key: &str, context: &CommandContext) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match value {
        JsonValue::Number(value) => value
            .parse()
            .map_err(|error| invalid_json(context.clone(), &format!("field '{key}' {error}"))),
        _ => Err(invalid_json(
            context.clone(),
            &format!("field '{key}' must be a number"),
        )),
    }
}

fn parse_root(payload: &str, context: CommandContext) -> Result<JsonValue> {
    let mut parser = Parser::new(payload, context.clone());
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.is_finished() {
        Ok(value)
    } else {
        Err(invalid_json(
            context,
            "trailing characters after JSON value",
        ))
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    cursor: usize,
    context: CommandContext,
}

impl<'a> Parser<'a> {
    const fn new(payload: &'a str, context: CommandContext) -> Self {
        Self {
            bytes: payload.as_bytes(),
            cursor: 0,
            context,
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue> {
        self.skip_whitespace();
        match self.peek_byte() {
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b't') => self.parse_literal(b"true", JsonValue::Bool(true)),
            Some(b'f') => self.parse_literal(b"false", JsonValue::Bool(false)),
            Some(b'n') => self.parse_literal(b"null", JsonValue::Null),
            Some(b'-' | b'0'..=b'9') => Ok(JsonValue::Number(self.parse_number()?)),
            Some(_) => Err(self.error("unexpected value token")),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        self.expect_byte(b'[')?;
        let mut values = Vec::new();
        loop {
            self.skip_whitespace();
            if self.consume_if(b']') {
                return Ok(JsonValue::Array(values));
            }

            values.push(self.parse_value()?);
            self.skip_whitespace();
            if self.consume_if(b',') {
                continue;
            }
            self.expect_byte(b']')?;
            return Ok(JsonValue::Array(values));
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        self.expect_byte(b'{')?;
        let mut fields = Vec::new();
        loop {
            self.skip_whitespace();
            if self.consume_if(b'}') {
                return Ok(JsonValue::Object(fields));
            }

            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            let value = self.parse_value()?;
            fields.push((key, value));
            self.skip_whitespace();
            if self.consume_if(b',') {
                continue;
            }
            self.expect_byte(b'}')?;
            return Ok(JsonValue::Object(fields));
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        self.expect_byte(b'"')?;
        let mut output = String::new();
        while let Some(byte) = self.next_byte() {
            match byte {
                b'"' => return Ok(output),
                b'\\' => output.push(self.parse_escape()?),
                0x00..=0x1f => return Err(self.error("control character in string")),
                _ => output.push(char::from(byte)),
            }
        }

        Err(self.error("unterminated string"))
    }

    fn parse_escape(&mut self) -> Result<char> {
        match self.next_byte() {
            Some(b'"') => Ok('"'),
            Some(b'\\') => Ok('\\'),
            Some(b'/') => Ok('/'),
            Some(b'b') => Ok('\u{0008}'),
            Some(b'f') => Ok('\u{000c}'),
            Some(b'n') => Ok('\n'),
            Some(b'r') => Ok('\r'),
            Some(b't') => Ok('\t'),
            Some(b'u') => self.parse_unicode_escape(),
            Some(_) => Err(self.error("invalid string escape")),
            None => Err(self.error("unterminated string escape")),
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char> {
        let mut value = 0_u32;
        for _ in 0..4 {
            let Some(byte) = self.next_byte() else {
                return Err(self.error("unterminated unicode escape"));
            };
            let Some(digit) = char::from(byte).to_digit(16) else {
                return Err(self.error("invalid unicode escape"));
            };
            value = (value << 4) + digit;
        }

        char::from_u32(value).ok_or_else(|| self.error("invalid unicode scalar"))
    }

    fn parse_number(&mut self) -> Result<String> {
        let start = self.cursor;
        self.consume_if(b'-');
        match self.next_byte() {
            Some(b'0') => {
                if matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    return Err(self.error("leading zero in number"));
                }
            }
            Some(b'1'..=b'9') => _ = self.consume_digits(),
            Some(_) | None => return Err(self.error("expected number digit")),
        }

        if self.consume_if(b'.') && !self.consume_digits() {
            return Err(self.error("expected fractional digit"));
        }
        if self.consume_if(b'e') || self.consume_if(b'E') {
            let _ = self.consume_if(b'+') || self.consume_if(b'-');
            if !self.consume_digits() {
                return Err(self.error("expected exponent digit"));
            }
        }

        let bytes = &self.bytes[start..self.cursor];
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|error| self.error(&format!("invalid number: {error}")))
    }

    fn parse_literal(&mut self, literal: &[u8], value: JsonValue) -> Result<JsonValue> {
        for expected in literal {
            self.expect_byte(*expected)?;
        }
        Ok(value)
    }

    fn skip_whitespace(&mut self) {
        self.cursor += self.bytes[self.cursor..]
            .iter()
            .take_while(|&&byte| matches!(byte, b' ' | b'\n' | b'\r' | b'\t'))
            .count();
    }

    fn consume_digits(&mut self) -> bool {
        let count = self.bytes[self.cursor..]
            .iter()
            .take_while(|&&byte| byte.is_ascii_digit())
            .count();
        self.cursor += count;
        count > 0
    }

    fn expect_byte(&mut self, expected: u8) -> Result<()> {
        if self.consume_if(expected) {
            Ok(())
        } else {
            Err(self.error(&format!("expected '{}'", char::from(expected))))
        }
    }

    fn consume_if(&mut self, expected: u8) -> bool {
        if self.peek_byte() == Some(expected) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        let byte = self.peek_byte()?;
        self.cursor = self.cursor.saturating_add(1);
        Some(byte)
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.cursor).copied()
    }

    const fn is_finished(&self) -> bool {
        self.cursor == self.bytes.len()
    }

    fn error(&self, message: &str) -> HyprlandError {
        invalid_json(
            self.context.clone(),
            &format!("{message} at byte {}", self.cursor),
        )
    }
}

fn invalid_json(context: CommandContext, message: &str) -> HyprlandError {
    HyprlandError::new(
        context,
        HyprlandErrorKind::InvalidJson {
            message: message.to_owned(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{JsonValue, Parser, parse_root};
    use crate::{CommandContext, HyprlandErrorKind};

    #[test]
    fn rejects_unescaped_string_control_characters() {
        for payload in [
            "\"bad\u{0}string\"",
            "\"bad\u{1f}string\"",
            "\"bad\nstring\"",
        ] {
            assert_invalid_json(payload, "control character in string");
        }
    }

    #[test]
    fn decodes_string_escapes_and_unicode_hex() {
        assert_eq!(
            parse_value(r#""quote\" slash\/ backslash\\ newline\n tab\t""#),
            JsonValue::String("quote\" slash/ backslash\\ newline\n tab\t".to_owned())
        );
        assert_eq!(
            parse_value(r#""\u0048\u0079\u0070\u0072\u0031""#),
            JsonValue::String("Hypr1".to_owned())
        );

        for payload in [r#""\u12""#, r#""\u12xz""#] {
            assert_invalid_json(payload, "unicode escape");
        }
    }

    #[test]
    fn enforces_json_number_boundaries() {
        for payload in [
            "0", "-0", "42", "-42", "3.14", "-3.14", "6e7", "6E-7", "6e+7",
        ] {
            assert_eq!(
                parse_value(payload),
                JsonValue::Number(payload.to_owned()),
                "{payload}"
            );
        }

        for payload in ["-", "-.", "01", "-01", "1.", "1e", "1e+", "1e-"] {
            assert_invalid_json(payload, "");
        }
    }

    #[test]
    fn root_parser_rejects_trailing_non_whitespace() {
        assert_eq!(parse_value(" [] \n\t"), JsonValue::Array(Vec::new()));
        assert_invalid_json("[] []", "trailing characters");
    }

    #[test]
    fn cursor_helpers_advance_to_completion() {
        let mut parser = Parser::new(" \n\t123x", context());
        parser.skip_whitespace();
        assert_eq!(parser.cursor, 3);
        assert!(parser.consume_digits());
        assert_eq!(parser.cursor, 6);
        assert!(!parser.consume_digits());
        assert_eq!(parser.next_byte(), Some(b'x'));
        assert!(parser.is_finished());
        assert_eq!(parser.next_byte(), None);
        assert!(parser.is_finished());
    }

    fn parse_value(payload: &str) -> JsonValue {
        match parse_root(payload, context()) {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        }
    }

    fn assert_invalid_json(payload: &str, message_part: &str) {
        let error = match parse_root(payload, context()) {
            Ok(value) => panic!("expected invalid JSON for {payload:?}, got {value:?}"),
            Err(error) => error,
        };
        assert!(matches!(
            error.kind(),
            HyprlandErrorKind::InvalidJson { message }
                if message.contains(message_part)
        ));
    }

    fn context() -> CommandContext {
        CommandContext::parser("test")
    }
}
