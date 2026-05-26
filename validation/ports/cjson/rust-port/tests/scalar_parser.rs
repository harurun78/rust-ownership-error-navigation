use cjson_rust_port::{parse_scalar, JsonEditError, JsonPathSegment, JsonValue};

use JsonPathSegment::{Index, Key};

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
fn parses_empty_arrays() {
    assert_eq!(parse_scalar("[]"), Ok(JsonValue::Array(Vec::new())));
    assert_eq!(parse_scalar("[ \n\t ]"), Ok(JsonValue::Array(Vec::new())));
}

#[test]
fn parses_mixed_scalar_arrays() {
    assert_eq!(
        parse_scalar(r#"[null, true, false, 42, "cat"]"#),
        Ok(JsonValue::Array(vec![
            JsonValue::Null,
            JsonValue::Bool(true),
            JsonValue::Bool(false),
            JsonValue::Number(42.0),
            JsonValue::String(String::from("cat")),
        ]))
    );
}

#[test]
fn parses_nested_arrays() {
    assert_eq!(
        parse_scalar("[1, [2, [3]]]"),
        Ok(JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Array(vec![
                JsonValue::Number(2.0),
                JsonValue::Array(vec![JsonValue::Number(3.0)]),
            ]),
        ]))
    );
}

#[test]
fn rejects_malformed_arrays() {
    assert!(parse_scalar("[1,]").is_err());
    assert!(parse_scalar("[1, 2").is_err());
}

#[test]
fn parses_empty_objects() {
    assert_eq!(parse_scalar("{}"), Ok(JsonValue::Object(Vec::new())));
    assert_eq!(parse_scalar("{ \n\t }"), Ok(JsonValue::Object(Vec::new())));
}

#[test]
fn parses_object_scalar_values() {
    assert_eq!(
        parse_scalar(r#"{"name":"cjson","ok":true,"count":3}"#),
        Ok(JsonValue::Object(vec![
            (
                String::from("name"),
                JsonValue::String(String::from("cjson"))
            ),
            (String::from("ok"), JsonValue::Bool(true)),
            (String::from("count"), JsonValue::Number(3.0)),
        ]))
    );
}

#[test]
fn parses_nested_arrays_and_objects() {
    assert_eq!(
        parse_scalar(r#"{"items":[{"id":1}, []], "meta":{"ready":false}}"#),
        Ok(JsonValue::Object(vec![
            (
                String::from("items"),
                JsonValue::Array(vec![
                    JsonValue::Object(vec![(String::from("id"), JsonValue::Number(1.0))]),
                    JsonValue::Array(Vec::new()),
                ]),
            ),
            (
                String::from("meta"),
                JsonValue::Object(vec![(String::from("ready"), JsonValue::Bool(false))]),
            ),
        ]))
    );
}

#[test]
fn rejects_malformed_objects() {
    assert!(parse_scalar(r#"{"key" true}"#).is_err());
    assert!(parse_scalar(r#"{"key": true,}"#).is_err());
    assert!(parse_scalar(r#"{key: true}"#).is_err());
}

#[test]
fn rejects_excessive_recursion_depth() {
    let input = format!("{}null{}", "[".repeat(130), "]".repeat(130));

    assert!(parse_scalar(&input).is_err());
}

#[test]
fn appends_values_to_arrays() {
    let mut value = JsonValue::Array(vec![JsonValue::Null]);

    assert_eq!(value.append_array(JsonValue::Bool(true)), Ok(()));
    assert_eq!(
        value,
        JsonValue::Array(vec![JsonValue::Null, JsonValue::Bool(true)])
    );
}

#[test]
fn rejects_array_append_on_non_arrays() {
    let mut value = JsonValue::Null;

    assert_eq!(
        value.append_array(JsonValue::Bool(true)),
        Err(JsonEditError::NotArray)
    );
    assert_eq!(value, JsonValue::Null);
}

#[test]
fn inserts_and_replaces_object_members() {
    let mut value = JsonValue::Object(vec![(String::from("name"), JsonValue::Null)]);

    assert_eq!(
        value.insert_object_member(String::from("enabled"), JsonValue::Bool(true)),
        Ok(None)
    );
    assert_eq!(
        value.insert_object_member(
            String::from("name"),
            JsonValue::String(String::from("cjson"))
        ),
        Ok(Some(JsonValue::Null))
    );
    assert_eq!(
        value,
        JsonValue::Object(vec![
            (
                String::from("name"),
                JsonValue::String(String::from("cjson"))
            ),
            (String::from("enabled"), JsonValue::Bool(true)),
        ])
    );
}

#[test]
fn rejects_object_insert_on_non_objects() {
    let mut value = JsonValue::Array(Vec::new());

    assert_eq!(
        value.insert_object_member(String::from("key"), JsonValue::Null),
        Err(JsonEditError::NotObject)
    );
    assert_eq!(value, JsonValue::Array(Vec::new()));
}

#[test]
fn detaches_array_items_by_index() {
    let mut value = JsonValue::Array(vec![
        JsonValue::String(String::from("first")),
        JsonValue::String(String::from("second")),
    ]);

    assert_eq!(
        value.detach_array_item(0),
        Ok(Some(JsonValue::String(String::from("first"))))
    );
    assert_eq!(
        value,
        JsonValue::Array(vec![JsonValue::String(String::from("second"))])
    );
}

#[test]
fn reports_missing_array_items_and_non_arrays() {
    let mut array = JsonValue::Array(Vec::new());
    let mut scalar = JsonValue::Bool(false);

    assert_eq!(array.detach_array_item(4), Ok(None));
    assert_eq!(scalar.detach_array_item(0), Err(JsonEditError::NotArray));
}

#[test]
fn detaches_object_members_by_key() {
    let mut value = JsonValue::Object(vec![
        (String::from("keep"), JsonValue::Bool(true)),
        (String::from("take"), JsonValue::Number(7.0)),
    ]);

    assert_eq!(
        value.detach_object_member("take"),
        Ok(Some(JsonValue::Number(7.0)))
    );
    assert_eq!(
        value,
        JsonValue::Object(vec![(String::from("keep"), JsonValue::Bool(true))])
    );
}

#[test]
fn reports_missing_object_members_and_non_objects() {
    let mut object = JsonValue::Object(Vec::new());
    let mut scalar = JsonValue::String(String::from("text"));

    assert_eq!(object.detach_object_member("missing"), Ok(None));
    assert_eq!(
        scalar.detach_object_member("missing"),
        Err(JsonEditError::NotObject)
    );
}

#[test]
fn finds_nested_values_by_path() {
    let value = parse_scalar(r#"{"items":[{"name":"first"},{"name":"second"}]}"#).unwrap();

    assert_eq!(
        value.get_path(&[Key("items"), Index(1), Key("name")]),
        Some(&JsonValue::String(String::from("second")))
    );
    assert_eq!(value.get_path(&[]), Some(&value));
}

#[test]
fn reports_missing_paths() {
    let value = parse_scalar(r#"{"items":[{"name":"first"}]}"#).unwrap();

    assert_eq!(value.get_path(&[Key("items"), Index(4)]), None);
    assert_eq!(value.get_path(&[Key("missing")]), None);
}

#[test]
fn mutates_nested_values_by_path() {
    let mut value = parse_scalar(r#"{"items":[{"done":false}]}"#).unwrap();

    let target = value.get_path_mut(&[Key("items"), Index(0), Key("done")]);
    assert_eq!(target, Some(&mut JsonValue::Bool(false)));

    if let Some(slot) = value.get_path_mut(&[Key("items"), Index(0), Key("done")]) {
        *slot = JsonValue::Bool(true);
    }

    assert_eq!(
        value.get_path(&[Key("items"), Index(0), Key("done")]),
        Some(&JsonValue::Bool(true))
    );
}

#[test]
fn replaces_nested_values_and_returns_old_value() {
    let mut value = parse_scalar(r#"{"items":[{"count":1}]}"#).unwrap();

    assert_eq!(
        value.replace_at_path(
            &[Key("items"), Index(0), Key("count")],
            JsonValue::Number(2.0)
        ),
        Some(JsonValue::Number(1.0))
    );
    assert_eq!(
        value.get_path(&[Key("items"), Index(0), Key("count")]),
        Some(&JsonValue::Number(2.0))
    );
}

#[test]
fn reports_non_container_paths_without_mutating() {
    let mut value = parse_scalar(r#"{"items":[true]}"#).unwrap();

    assert_eq!(value.get_path(&[Key("items"), Index(0), Key("name")]), None);
    assert_eq!(
        value.replace_at_path(
            &[Key("items"), Index(0), Key("name")],
            JsonValue::String(String::from("ignored"))
        ),
        None
    );
    assert_eq!(
        value.get_path(&[Key("items"), Index(0)]),
        Some(&JsonValue::Bool(true))
    );
}

#[test]
fn prints_scalar_values_compactly() {
    assert_eq!(JsonValue::Null.to_compact_string(), "null");
    assert_eq!(JsonValue::Bool(true).to_compact_string(), "true");
    assert_eq!(JsonValue::Bool(false).to_compact_string(), "false");
    assert_eq!(JsonValue::Number(42.5).to_compact_string(), "42.5");
    assert_eq!(
        JsonValue::String(String::from("cat")).to_compact_string(),
        r#""cat""#
    );
}

#[test]
fn escapes_strings_for_compact_printing() {
    let value = JsonValue::String(String::from(
        "quote: \" slash: / backslash: \\ controls: \u{0008}\u{000c}\n\r\t\u{0001}",
    ));

    assert_eq!(
        value.to_compact_string(),
        r#""quote: \" slash: / backslash: \\ controls: \b\f\n\r\t\u0001""#
    );
}

#[test]
fn prints_arrays_and_objects_compactly() {
    let value = JsonValue::Object(vec![
        (
            String::from("items"),
            JsonValue::Array(vec![
                JsonValue::Null,
                JsonValue::Bool(true),
                JsonValue::String(String::from("cat")),
            ]),
        ),
        (String::from("count"), JsonValue::Number(3.0)),
    ]);

    assert_eq!(
        value.to_compact_string(),
        r#"{"items":[null,true,"cat"],"count":3}"#
    );
}

#[test]
fn round_trips_parsed_values_through_compact_printing() {
    let inputs = [
        r#"{"name":"cjson","items":[null,true,false,12.5],"nested":{"key":"value"}}"#,
        r#"["escaped\ntext",{"unicode":"猫"}]"#,
        r#"{"control":"\u0001\b\f\n\r\t"}"#,
    ];

    for input in inputs {
        let parsed = parse_scalar(input).unwrap();
        let printed = parsed.to_compact_string();
        assert_eq!(parse_scalar(&printed), Ok(parsed));
    }
}
