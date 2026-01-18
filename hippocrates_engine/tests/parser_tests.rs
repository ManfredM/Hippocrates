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
