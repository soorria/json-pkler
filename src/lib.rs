#[derive(Debug, Clone, PartialEq)]
pub enum JSONValue {
    String(String),
    Number(f64),
    Object(Box<Vec<(String, JSONValue)>>),
    Array(Box<Vec<JSONValue>>),
    True,
    False,
    Null,
}

pub struct JSONBuilder {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseJSONError(String);

/// Given the chars, and position of the starting '"', returns
/// the index of the end quote and the found string
fn parse_json_string(chars: &Vec<char>, from: usize) -> Result<(usize, String), ParseJSONError> {
    let mut i = from + 1;
    let mut string_end_found = false;

    while let Some(ch) = chars.get(i) {
        if ch == &'"' && chars.get(i - 1) != Some(&'\\') {
            string_end_found = true;
            break;
        }

        i += 1;
    }

    if !string_end_found {
        return Err(ParseJSONError(String::from(
            "Missing end quotes for string",
        )));
    }

    return Ok((i, chars[from + 1..i].iter().collect()));
}

fn parse_json_number(chars: &Vec<char>, from: usize) -> Result<(usize, f64), ParseJSONError> {
    // If the first char is a minus sign, let's just skip for simplicity
    // in the loop below
    let mut i = match chars.get(from) {
        Some(ch) if ch == &'-' => from + 1,
        _ => from,
    };

    let mut parsing_exponent = false;
    let mut found_decimal = false;
    let mut found_exponent_sign = false;

    while let Some(ch) = chars.get(i) {
        if ch.is_numeric() {
            i += 1;
            continue;
        }

        if ch == &'.' && !found_decimal {
            found_decimal = true;
        } else if ch == &'e' || ch == &'E' {
            parsing_exponent = true;
            found_decimal = false;
        } else if ch == &'+' || ch == &'-' && parsing_exponent && !found_exponent_sign {
            found_exponent_sign = true;
        } else {
            break;
        }

        i += 1;
    }

    let parsed = chars[from..i]
        .iter()
        .collect::<String>()
        .parse()
        .map_err(|_| ParseJSONError("Invalid number".to_string()))?;

    return Ok((i - 1, parsed));
}

fn parse_json_literal(
    chars: &Vec<char>,
    from: usize,
    literal: &str,
) -> Result<usize, ParseJSONError> {
    let text = (from..from + literal.len())
        .filter_map(|i| chars.get(i))
        .collect::<String>();
    let is_null = text == literal;
    return if is_null {
        Ok(from + literal.len() - 1)
    } else {
        Err(ParseJSONError(
            format!("Expected {literal} but got {text}").to_string(),
        ))
    };
}

pub fn parse_json(string: &str) -> Result<JSONValue, ParseJSONError> {
    let mut i = 0;
    let chars = string.chars().collect::<Vec<char>>();

    let mut output: Option<JSONValue> = None;

    while let Some(ch) = chars.get(i) {
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        if ch == &'"' {
            let (end_index, parsed_string) = parse_json_string(&chars, i)?;
            output = Some(JSONValue::String(parsed_string));
            i = end_index;
        } else if ch.is_numeric() || ch == &'-' {
            let (end_index, parsed_number) = parse_json_number(&chars, i)?;
            output = Some(JSONValue::Number(parsed_number));
            i = end_index;
        } else if ch == &'n' {
            let end_index = parse_json_literal(&chars, i, "null")?;
            output = Some(JSONValue::Null);
            i = end_index;
        } else if ch == &'t' {
            let end_index = parse_json_literal(&chars, i, "true")?;
            output = Some(JSONValue::True);
            i = end_index;
        } else if ch == &'f' {
            let end_index = parse_json_literal(&chars, i, "false")?;
            output = Some(JSONValue::False);
            i = end_index;
        }

        i += 1;
    }

    return output.ok_or(ParseJSONError("Nothing found in ur json".to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_string_simple_string() {
        assert_eq!(
            parse_json_string(&r#"   "hello, world!""#.chars().collect(), 3),
            Ok((17, "hello, world!".to_string()))
        );
    }

    #[test]
    fn parse_json_string_with_escapes() {
        assert_eq!(
            parse_json_string(&r#""hello\", world!""#.chars().collect(), 0),
            Ok((16, r#"hello\", world!"#.to_string()))
        );
    }

    #[test]
    fn parse_json_string_incomplete_string_err() {
        assert_eq!(
            parse_json_string(&r#""hello, world!"#.chars().collect(), 0),
            Err(ParseJSONError("Missing end quotes for string".to_string()))
        );
    }

    #[test]
    #[ignore]
    fn parse_json_string_with_unicode() {
        assert_eq!(
            parse_json_string(&r#""\u0928""#.chars().collect(), 0),
            Ok((7, "न".to_string()))
        )
    }

    #[test]
    fn parse_json_just_string() {
        assert_eq!(
            parse_json(r#""hello, world!""#),
            Ok(JSONValue::String("hello, world!".to_string()))
        );
    }

    #[test]
    fn parse_json_number_1() {
        assert_eq!(
            parse_json_number(&r#"-1.2e+3"#.chars().collect(), 0),
            Ok((6, -1200f64))
        );
    }

    #[test]
    fn parse_json_number_2() {
        assert_eq!(
            parse_json_number(&r#"-1.2E-3,"#.chars().collect(), 0),
            Ok((6, -0.0012f64))
        );
    }

    #[test]
    fn parse_json_just_number() {
        assert_eq!(parse_json(&r#"-1.2e+3"#), Ok(JSONValue::Number(-1200f64)));
    }

    #[test]
    fn parse_json_null() {
        assert_eq!(parse_json("null"), Ok(JSONValue::Null));
    }

    #[test]
    fn parse_json_true() {
        assert_eq!(parse_json("true"), Ok(JSONValue::True));
    }

    #[test]
    fn parse_json_false() {
        assert_eq!(parse_json("false"), Ok(JSONValue::False));
    }

    #[test]
    #[ignore]
    fn parse_json_empty_object() {
        assert_eq!(parse_json("{}"), Ok(JSONValue::Object(Box::new(vec![]))))
    }
}
