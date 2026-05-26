use cjson_rust_port::{parse_scalar, JsonValue, ParseError};

#[test]
fn parses_literals() {
    assert_eq!(parse_scalar("null"), Ok(JsonValue::Null));
    assert_eq!(parse_scalar("true"), Ok(JsonValue::Bool(true)));
    assert_eq!(parse_scalar("false"), Ok(JsonValue::Bool(false)));
    assert_eq!(parse_scalar(" \n\ttrue\r "), Ok(JsonValue::Bool(true)));
}

#[test]
fn parses_numbers() {
    let cases = [
        ("0", 0.0),
        ("0.0", 0.0),
        ("-0", -0.0),
        ("42", 42.0),
        ("-17", -17.0),
        ("3.1415", 3.1415),
        ("-10.25", -10.25),
        ("1e3", 1000.0),
        ("1E3", 1000.0),
        ("1e-3", 0.001),
        ("-2.5E+2", -250.0),
    ];

    for (input, expected) in cases {
        assert_eq!(parse_scalar(input), Ok(JsonValue::Number(expected)));
    }
}

#[test]
fn parses_large_numbers() {
    match parse_scalar("123456789012345678901234567890") {
        Ok(JsonValue::Number(number)) => assert!(number.is_finite()),
        other => panic!("unexpected parse result: {other:?}"),
    }

    assert_eq!(
        parse_scalar("1.2345678901234568e+30"),
        Ok(JsonValue::Number(1.2345678901234568e30))
    );
}

#[test]
fn parses_strings_and_common_escapes() {
    assert_eq!(parse_scalar("\"\""), Ok(JsonValue::String(String::new())));
    assert_eq!(
        parse_scalar("\"hello\""),
        Ok(JsonValue::String(String::from("hello")))
    );
    assert_eq!(
        parse_scalar(r#""\"\\\/\b\f\n\r\t""#),
        Ok(JsonValue::String(String::from(
            "\"\\/\u{0008}\u{000c}\n\r\t"
        )))
    );
}

#[test]
fn parses_unicode_escapes() {
    assert_eq!(
        parse_scalar(r#""\u20AC""#),
        Ok(JsonValue::String(String::from("€")))
    );
    assert_eq!(
        parse_scalar(r#""\u732b""#),
        Ok(JsonValue::String(String::from("猫")))
    );
    assert_eq!(
        parse_scalar(r#""\uD834\uDD1E""#),
        Ok(JsonValue::String(String::from("𝄞")))
    );
}

#[test]
fn rejects_invalid_scalar_inputs() {
    assert!(parse_scalar("").is_err());
    assert!(parse_scalar("not-json").is_err());
    assert!(parse_scalar("\"unterminated").is_err());
    assert!(parse_scalar(r#""\z""#).is_err());
    assert!(parse_scalar("\"\\").is_err());
    assert!(parse_scalar("01").is_err());
    assert!(parse_scalar("1e").is_err());
    assert!(parse_scalar("true false").is_err());
    assert!(parse_scalar(r#""\uD834x""#).is_err());
}

#[test]
fn rejects_arrays_and_objects_in_phase_one() {
    assert_eq!(
        parse_scalar("[]"),
        Err(ParseError::Unsupported {
            feature: "array",
            pos: 0,
        })
    );
    assert_eq!(
        parse_scalar("{}"),
        Err(ParseError::Unsupported {
            feature: "object",
            pos: 0,
        })
    );
}
