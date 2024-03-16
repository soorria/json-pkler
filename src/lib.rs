#[derive(Debug, Clone, PartialEq)]
pub enum JSONValue {
    String(String),
    Number(f64),
    Object(Vec<(String, JSONValue)>),
    Array(Vec<JSONValue>),
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

        if ch == &'.' && !found_decimal && !parsing_exponent {
            found_decimal = true;
        } else if ch == &'e' || ch == &'E' {
            parsing_exponent = true;
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

fn trim_whitespace(chars: &Vec<char>, from: usize) -> usize {
    let mut i = from;
    while matches!(chars.get(i), Some(ch) if ch.is_whitespace()) {
        i += 1;
    }
    return i;
}

fn parse_json_array(
    chars: &Vec<char>,
    from: usize,
) -> Result<(usize, Vec<JSONValue>), ParseJSONError> {
    let mut i = from + 1;

    let mut output = vec![];
    let mut array_should_end = false;
    let mut is_ok_for_array_to_end = true;

    while let Some(ch) = chars.get(i) {
        i = trim_whitespace(chars, i);

        if ch == &']' && is_ok_for_array_to_end {
            break;
        } else if array_should_end {
            return Err(ParseJSONError("Expected ']' to end array".to_string()));
        } else if ch == &',' {
            return Err(ParseJSONError("Unexpected comma".to_string()));
        }

        let (end_index, json_value) = parse_json_value(chars, i)?;
        output.push(json_value);
        i = end_index + 1;
        i = trim_whitespace(chars, i);

        // if the next char is a comma, we expect another item in this array
        // so we should error if the array just ends
        if chars.get(i) == Some(&',') {
            i += 1;
            is_ok_for_array_to_end = false;
        } else {
            array_should_end = true;
            is_ok_for_array_to_end = true;
        }
    }

    return Ok((i, output));
}

pub fn parse_json_value(
    chars: &Vec<char>,
    from: usize,
) -> Result<(usize, JSONValue), ParseJSONError> {
    let mut i = from;

    i = trim_whitespace(chars, i);

    let ch = chars.get(i);

    let (value_end_index, json_value) = match ch {
        // Strings
        Some(&'"') => {
            let (end_index, parsed_string) = parse_json_string(&chars, i)?;
            (end_index, JSONValue::String(parsed_string))
        }

        // null
        Some(&'n') => {
            let end_index = parse_json_literal(&chars, i, "null")?;
            (end_index, JSONValue::Null)
        }

        // booleans
        Some(&'t') => {
            let end_index = parse_json_literal(&chars, i, "true")?;
            (end_index, JSONValue::True)
        }
        Some(&'f') => {
            let end_index = parse_json_literal(&chars, i, "false")?;
            (end_index, JSONValue::False)
        }

        // numbers
        Some(ch) if ch.is_numeric() || ch == &'-' => {
            let (end_index, parsed_number) = parse_json_number(&chars, i)?;
            (end_index, JSONValue::Number(parsed_number))
        }

        Some(&'[') => {
            let (end_index, parsed_array) = parse_json_array(chars, i)?;
            (end_index, JSONValue::Array(parsed_array))
        }

        _ => return Err(ParseJSONError("No JSON value found".to_string())),
    };

    i = trim_whitespace(chars, value_end_index);

    return Ok((i, json_value));
}

pub fn parse_json(string: &str) -> Result<JSONValue, ParseJSONError> {
    let chars = string.chars().collect::<Vec<char>>();

    let (_end_index, json_value) = parse_json_value(&chars, 0)?;

    return Ok(json_value);
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
    fn parse_json_array_empty_array() {
        assert_eq!(
            parse_json_array(&"[]".chars().collect(), 0),
            Ok((1, vec![]))
        )
    }

    #[test]
    fn parse_json_array_numbers_array() {
        assert_eq!(
            parse_json_array(&"[ 1 , 2 , 3 ]".chars().collect(), 0),
            Ok((
                12,
                vec![
                    JSONValue::Number(1f64),
                    JSONValue::Number(2f64),
                    JSONValue::Number(3f64),
                ]
            ))
        )
    }

    #[test]
    fn parse_json_array_trailing_comma() {
        assert_eq!(
            parse_json_array(&"[1, 2,]".chars().collect(), 0),
            Err(ParseJSONError("No JSON value found".to_string()))
        )
    }

    #[test]
    fn parse_json_array_double_comma() {
        assert_eq!(
            parse_json_array(&"[1, 2,,]".chars().collect(), 0),
            Err(ParseJSONError("Unexpected comma".to_string()))
        )
    }

    #[test]
    fn parse_json_array_missing_comma() {
        assert_eq!(
            parse_json_array(&"[1, 2  3]".chars().collect(), 0),
            Err(ParseJSONError("Expected ']' to end array".to_string()))
        )
    }

    #[test]
    fn parse_json_array_nested_array() {
        assert_eq!(
            parse_json_array(&"[1, [2, [3]]]".chars().collect(), 0),
            Ok((
                12,
                vec![
                    JSONValue::Number(1.0),
                    JSONValue::Array(vec![
                        JSONValue::Number(2.0),
                        JSONValue::Array(vec![JSONValue::Number(3.0),])
                    ])
                ]
            ))
        )
    }

    #[test]
    fn parse_json_just_empty_array() {
        assert_eq!(parse_json("[]"), Ok(JSONValue::Array(vec![])))
    }

    #[test]
    #[ignore]
    fn parse_json_empty_object() {
        assert_eq!(parse_json("{}"), Ok(JSONValue::Object(vec![])))
    }
}
