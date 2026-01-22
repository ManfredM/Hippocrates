// Spec §2, §3.1: identifiers, strings, and operator restrictions.

use hippocrates_engine::parser;

// REQ-2-04: block openings require a newline and indented block.
#[test]
fn spec_block_requires_newline_after_colon() {
    let input = r#"
<plan> is a plan: during plan:
    show message "Hi".
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for inline block after ':'");
}

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

// REQ-3.1-03: inline ':' forms are only allowed where explicitly shown.
#[test]
fn spec_inline_colon_requires_block() {
    let input = r#"
<temp> is a number:
    question: ask "Temp?".
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for inline property block");
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

// REQ-3.2-05: numeric literals must include units.
#[test]
fn spec_unitless_numeric_literal_fails() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 10 kg.

<plan> is a plan:
    during plan:
        <val> = 5.
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for unitless numeric literal");
}

// REQ-3.6-04: statements inside blocks must terminate with a period.
#[test]
fn spec_block_statements_require_period() {
    let input = r#"
<plan> is a plan:
    during plan:
        show message "Hi"
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for missing period in block statement");
}

// REQ-3.6-05: blocks must be introduced with a colon.
#[test]
fn spec_blocks_require_colon() {
    let input = r#"
<plan> is a plan:
    during plan
        show message "Hi".
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for missing ':' on block");
}
