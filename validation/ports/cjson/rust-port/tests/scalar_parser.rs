use cjson_rust_port::{
    minify_json, parse_json_pointer, parse_scalar, JsonEditError, JsonPathSegment,
    JsonPointerError, JsonValue, MinifyError,
};

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
fn detaches_nested_array_items_by_path() {
    let mut value = parse_scalar(r#"{"items":["first","second","third"]}"#).unwrap();

    assert_eq!(
        value.detach_at_path(&[Key("items"), Index(1)]),
        Ok(Some(JsonValue::String(String::from("second"))))
    );
    assert_eq!(
        value.get_path(&[Key("items")]),
        Some(&JsonValue::Array(vec![
            JsonValue::String(String::from("first")),
            JsonValue::String(String::from("third")),
        ]))
    );
}

#[test]
fn detaches_nested_object_members_by_path() {
    let mut value = parse_scalar(r#"{"meta":{"keep":true,"take":7}}"#).unwrap();

    assert_eq!(
        value.detach_at_path(&[Key("meta"), Key("take")]),
        Ok(Some(JsonValue::Number(7.0)))
    );
    assert_eq!(
        value.get_path(&[Key("meta")]),
        Some(&JsonValue::Object(vec![(
            String::from("keep"),
            JsonValue::Bool(true)
        )]))
    );
}

#[test]
fn reports_missing_terminal_items_for_path_detach() {
    let mut value = parse_scalar(r#"{"items":[null],"meta":{}}"#).unwrap();

    assert_eq!(value.detach_at_path(&[Key("items"), Index(3)]), Ok(None));
    assert_eq!(
        value.detach_at_path(&[Key("meta"), Key("missing")]),
        Ok(None)
    );
}

#[test]
fn rejects_missing_parent_and_empty_path_for_detach() {
    let mut value = parse_scalar(r#"{"items":[null]}"#).unwrap();

    assert_eq!(
        value.detach_at_path(&[Key("missing"), Index(0)]),
        Err(JsonEditError::MissingPath)
    );
    assert_eq!(value.detach_at_path(&[]), Err(JsonEditError::EmptyPath));
    assert_eq!(value.get_path(&[]), Some(&value));
}

#[test]
fn rejects_non_container_parent_and_terminal_mismatch_for_detach() {
    let mut value = parse_scalar(r#"{"items":[true],"meta":{}}"#).unwrap();

    assert_eq!(
        value.detach_at_path(&[Key("items"), Index(0), Key("name")]),
        Err(JsonEditError::NotObject)
    );
    assert_eq!(
        value.detach_at_path(&[Key("meta"), Index(0)]),
        Err(JsonEditError::NotArray)
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

#[test]
fn pretty_prints_scalar_values() {
    assert_eq!(JsonValue::Null.to_pretty_string(), "null");
    assert_eq!(JsonValue::Bool(true).to_pretty_string(), "true");
    assert_eq!(JsonValue::Number(42.5).to_pretty_string(), "42.5");
    assert_eq!(
        JsonValue::String(String::from("cat")).to_pretty_string(),
        r#""cat""#
    );
}

#[test]
fn pretty_prints_arrays_with_two_space_indentation() {
    let value = JsonValue::Array(vec![
        JsonValue::Null,
        JsonValue::Bool(true),
        JsonValue::Number(3.0),
    ]);

    assert_eq!(value.to_pretty_string(), "[\n  null,\n  true,\n  3\n]");
    assert_eq!(JsonValue::Array(Vec::new()).to_pretty_string(), "[]");
}

#[test]
fn pretty_prints_nested_objects_and_arrays() {
    let value = parse_scalar(r#"{"items":[{"name":"first"},[]],"meta":{"ready":false}}"#).unwrap();

    assert_eq!(
        value.to_pretty_string(),
        "{\n  \"items\": [\n    {\n      \"name\": \"first\"\n    },\n    []\n  ],\n  \"meta\": {\n    \"ready\": false\n  }\n}"
    );
}

#[test]
fn pretty_printing_preserves_string_escaping() {
    let value = JsonValue::Object(vec![
        (
            String::from("quote\"key"),
            JsonValue::String(String::from("line\nquote \" slash / backslash \\")),
        ),
        (
            String::from("control"),
            JsonValue::String(String::from("\u{0001}\u{0008}\u{000c}\r\t")),
        ),
    ]);

    assert_eq!(
        value.to_pretty_string(),
        "{\n  \"quote\\\"key\": \"line\\nquote \\\" slash / backslash \\\\\",\n  \"control\": \"\\u0001\\b\\f\\r\\t\"\n}"
    );
}

#[test]
fn reports_value_type_predicates() {
    let values = [
        JsonValue::Null,
        JsonValue::Bool(true),
        JsonValue::Number(1.0),
        JsonValue::String(String::from("text")),
        JsonValue::Array(Vec::new()),
        JsonValue::Object(Vec::new()),
    ];

    assert!(values[0].is_null());
    assert!(values[1].is_bool());
    assert!(values[2].is_number());
    assert!(values[3].is_string());
    assert!(values[4].is_array());
    assert!(values[5].is_object());

    assert!(!values[0].is_bool());
    assert!(!values[1].is_number());
    assert!(!values[2].is_string());
    assert!(!values[3].is_array());
    assert!(!values[4].is_object());
    assert!(!values[5].is_null());
}

#[test]
fn returns_typed_accessors_for_matching_values() {
    let value = parse_scalar(r#"{"flag":true,"count":3,"name":"cjson","items":[null]}"#).unwrap();
    let object = value.as_object().unwrap();

    assert_eq!(object.len(), 4);
    assert_eq!(
        value.object_member("flag").and_then(JsonValue::as_bool),
        Some(true)
    );
    assert_eq!(
        value.object_member("count").and_then(JsonValue::as_number),
        Some(3.0)
    );
    assert_eq!(
        value.object_member("name").and_then(JsonValue::as_str),
        Some("cjson")
    );
    assert_eq!(
        value
            .object_member("items")
            .and_then(JsonValue::as_array)
            .and_then(|items| items.first()),
        Some(&JsonValue::Null)
    );
}

#[test]
fn returns_none_for_wrong_type_accessors() {
    let value = JsonValue::String(String::from("not a bool"));

    assert_eq!(value.as_bool(), None);
    assert_eq!(value.as_number(), None);
    assert_eq!(value.as_array(), None);
    assert_eq!(value.as_object(), None);
    assert_eq!(JsonValue::Bool(false).as_str(), None);
}

#[test]
fn finds_object_members_and_array_items() {
    let value = parse_scalar(r#"{"items":["first","second"],"enabled":false}"#).unwrap();

    assert_eq!(
        value
            .object_member("items")
            .and_then(|items| items.array_item(1)),
        Some(&JsonValue::String(String::from("second")))
    );
    assert_eq!(
        value.object_member("enabled"),
        Some(&JsonValue::Bool(false))
    );
}

#[test]
fn reports_missing_members_indexes_and_non_containers() {
    let value = parse_scalar(r#"{"items":[null]}"#).unwrap();

    assert_eq!(value.object_member("missing"), None);
    assert_eq!(
        value
            .object_member("items")
            .and_then(|items| items.array_item(5)),
        None
    );
    assert_eq!(JsonValue::Null.object_member("anything"), None);
    assert_eq!(JsonValue::Null.array_item(0), None);
}

#[test]
fn mutates_values_through_typed_accessors() {
    let mut value = parse_scalar(r#"{"items":[false],"name":"cjson"}"#).unwrap();

    if let Some(name) = value
        .object_member_mut("name")
        .and_then(JsonValue::as_string_mut)
    {
        name.push_str("-rust");
    }

    if let Some(flag) = value
        .object_member_mut("items")
        .and_then(|items| items.array_item_mut(0))
        .and_then(JsonValue::as_bool_mut)
    {
        *flag = true;
    }

    if let Some(items) = value
        .object_member_mut("items")
        .and_then(JsonValue::as_array_mut)
    {
        items.push(JsonValue::Number(2.0));
    }

    assert_eq!(
        value,
        parse_scalar(r#"{"items":[true,2],"name":"cjson-rust"}"#).unwrap()
    );
}

#[test]
fn minifies_insignificant_whitespace() {
    assert_eq!(
        minify_json(" { \n  \"items\" : [ true, null, 3 ] \t } "),
        Ok(String::from(r#"{"items":[true,null,3]}"#))
    );
}

#[test]
fn minify_preserves_whitespace_and_escapes_inside_strings() {
    assert_eq!(
        minify_json(r#"{ "text" : " a \n spaced \" string // not comment " }"#),
        Ok(String::from(
            r#"{"text":" a \n spaced \" string // not comment "}"#
        ))
    );
}

#[test]
fn minify_removes_line_and_block_comments() {
    let input = "{ // line comment\n \"a\": 1, /* block comment */ \"b\": [true] }";

    assert_eq!(
        minify_json(input),
        Ok(String::from(r#"{"a":1,"b":[true]}"#))
    );
}

#[test]
fn minify_reports_malformed_comments_and_strings() {
    assert_eq!(
        minify_json(r#"{"unterminated":"value"#),
        Err(MinifyError::UnterminatedString { pos: 16 })
    );
    assert_eq!(
        minify_json(r#"{"a":1 /* open"#),
        Err(MinifyError::UnterminatedBlockComment { pos: 7 })
    );
}

#[test]
fn applies_simple_object_merge_patch() {
    let mut value = parse_scalar(r#"{"title":"old","unchanged":true}"#).unwrap();

    value.apply_merge_patch(parse_scalar(r#"{"title":"new","count":2}"#).unwrap());

    assert_eq!(
        value,
        parse_scalar(r#"{"title":"new","unchanged":true,"count":2}"#).unwrap()
    );
}

#[test]
fn merge_patch_null_entries_delete_members() {
    let mut value = parse_scalar(r#"{"remove":false,"keep":"yes"}"#).unwrap();

    value.apply_merge_patch(parse_scalar(r#"{"remove":null,"missing":null}"#).unwrap());

    assert_eq!(value, parse_scalar(r#"{"keep":"yes"}"#).unwrap());
}

#[test]
fn applies_nested_object_merge_patch_recursively() {
    let mut value = parse_scalar(r#"{"meta":{"name":"old","keep":true},"items":[1]}"#).unwrap();

    value.apply_merge_patch(parse_scalar(r#"{"meta":{"name":"new","extra":3}}"#).unwrap());

    assert_eq!(
        value,
        parse_scalar(r#"{"meta":{"name":"new","keep":true,"extra":3},"items":[1]}"#).unwrap()
    );
}

#[test]
fn non_object_merge_patch_replaces_target() {
    let mut value = parse_scalar(r#"{"meta":{"name":"old"}}"#).unwrap();

    value.apply_merge_patch(JsonValue::Array(vec![JsonValue::Bool(true)]));

    assert_eq!(value, JsonValue::Array(vec![JsonValue::Bool(true)]));
}

#[test]
fn object_merge_patch_turns_non_object_target_into_object() {
    let mut value = JsonValue::String(String::from("not an object"));

    value.apply_merge_patch(parse_scalar(r#"{"created":true}"#).unwrap());

    assert_eq!(value, parse_scalar(r#"{"created":true}"#).unwrap());
}

#[test]
fn parses_empty_and_basic_json_pointers() {
    assert_eq!(parse_json_pointer(""), Ok(Vec::new()));
    assert_eq!(parse_json_pointer("/name"), Ok(vec![String::from("name")]));
    assert_eq!(
        parse_json_pointer("/items/0/name"),
        Ok(vec![
            String::from("items"),
            String::from("0"),
            String::from("name"),
        ])
    );
}

#[test]
fn decodes_json_pointer_escapes() {
    assert_eq!(
        parse_json_pointer("/a~1b/c~0d"),
        Ok(vec![String::from("a/b"), String::from("c~d")])
    );
}

#[test]
fn rejects_invalid_json_pointer_syntax() {
    assert_eq!(
        parse_json_pointer("name"),
        Err(JsonPointerError::InvalidPrefix)
    );
    assert_eq!(
        parse_json_pointer("/bad~2escape"),
        Err(JsonPointerError::InvalidEscape)
    );
    assert_eq!(
        parse_json_pointer("/dangling~"),
        Err(JsonPointerError::InvalidEscape)
    );
}

#[test]
fn resolves_json_pointer_paths() {
    let value =
        parse_scalar(r#"{"name":"root","items":[{"name":"first"}],"a/b":{"c~d":true}}"#).unwrap();

    assert_eq!(value.get_pointer(""), Ok(Some(&value)));
    assert_eq!(
        value.get_pointer("/name"),
        Ok(Some(&JsonValue::String(String::from("root"))))
    );
    assert_eq!(
        value.get_pointer("/items/0/name"),
        Ok(Some(&JsonValue::String(String::from("first"))))
    );
    assert_eq!(
        value.get_pointer("/a~1b/c~0d"),
        Ok(Some(&JsonValue::Bool(true)))
    );
}

#[test]
fn reports_json_pointer_missing_paths_and_array_parse_failures() {
    let value = parse_scalar(r#"{"items":[null],"0":"object key"}"#).unwrap();

    assert_eq!(value.get_pointer("/missing"), Ok(None));
    assert_eq!(value.get_pointer("/items/4"), Ok(None));
    assert_eq!(
        value.get_pointer("/items/not-a-number"),
        Err(JsonPointerError::InvalidArrayIndex)
    );
    assert_eq!(
        value.get_pointer("/0"),
        Ok(Some(&JsonValue::String(String::from("object key"))))
    );
}

#[test]
fn mutates_values_through_json_pointer_paths() {
    let mut value = parse_scalar(r#"{"items":[{"done":false}]}"#).unwrap();

    if let Some(target) = value.get_pointer_mut("/items/0/done").unwrap() {
        *target = JsonValue::Bool(true);
    }

    assert_eq!(
        value.get_pointer("/items/0/done"),
        Ok(Some(&JsonValue::Bool(true)))
    );
}
