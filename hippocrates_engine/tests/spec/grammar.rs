// Spec §2, §3.1: identifiers, strings, and operator restrictions.

use hippocrates_engine::parser;

// REQ-2-01: identifiers must use angle brackets.
#[test]
fn spec_identifiers_require_angle_brackets() {
    let input = r#"
Temperature is a number:
    valid values:
        0 kg ... 10 kg.
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for non-angled identifier");
}

// REQ-2-02: string literals must not contain angle brackets.
#[test]
fn spec_string_literal_rejects_angle_brackets() {
    let input = r#"
"test" is a plan:
《during plan:
    show message "Hello <world>".
》
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for string containing angle brackets");
}

// REQ-2-03: comparison operators are not supported; use ranges.
#[test]
fn spec_no_comparison_operators() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 10 kg.

<plan> is a plan:
    during plan:
        <val> < 5 kg.
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for comparison operator");
}
