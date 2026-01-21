use hippocrates_engine::parser;

#[test]
fn test_string_with_angle_brackets_fails() {
    let input = r#"
        "test" is a plan:
        《during plan:
            show message "Hello <world>".
        》
    "#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for string containing angle brackets");
}

#[test]
fn test_unitless_range_parsing() {
    let input = r#"
<val> is a number:
    valid values:
        0 ... 100
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for unitless range");
}

#[test]
fn test_custom_unit_quantity_parsing() {
    let input = r#"
<val> is a number:
    valid values:
        0 <points> ... 10 <points>
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_ok(), "Expected valid parse for custom unit quantity. Error: {:?}", result.err());
}
