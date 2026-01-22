use hippocrates_engine::{format_script, parser};

#[test]
fn formatter_inserts_missing_statement_newlines() {
    let input = r#"
<unit> is a unit:
  plural is <units>.

<value> is a number:
  valid values:
    0 <units> ... 1 <unit>.

<plan> is a plan:
  during plan:
    show message "First". show message "Second".
"#;

    let formatted = format_script(input).expect("Formatter failed");
    assert!(
        formatted.contains("show message \"First\".\n        show message \"Second\"."),
        "Expected newline between statements, got:\n{}",
        formatted
    );

    parser::parse_plan(&formatted).expect("Formatted output should parse");
}

#[test]
fn formatter_normalizes_indentation() {
    let input = r#"
<unit> is a unit:
  plural is <units>.

<value> is a number:
  valid values:
    0 <units> ... 10 <units>.

<plan> is a plan:
  during plan:
    assess <value>:
      0 <units> ... 10 <units>:
        show message "OK".
"#;

    let formatted = format_script(input).expect("Formatter failed");
    let expected_line = "        show message \"OK\".";
    assert!(
        formatted.contains(expected_line),
        "Expected normalized indentation, got:\n{}",
        formatted
    );

    parser::parse_plan(&formatted).expect("Formatted output should parse");
}

#[test]
fn formatter_preserves_message_recipient() {
    let input = r#"
<plan> is a plan:
  during plan:
    show message to <patient> "Hello":
      message expires after 1 day.
"#;

    let formatted = format_script(input).expect("Formatter failed");
    assert!(
        formatted.contains("show message to <patient> \"Hello\":"),
        "Expected recipient in output, got:\n{}",
        formatted
    );

    parser::parse_plan(&formatted).expect("Formatted output should parse");
}
