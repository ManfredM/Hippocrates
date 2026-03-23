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
  before plan:
    information "First". information "Second".
"#;

    let formatted = format_script(input).expect("Formatter failed");
    assert!(
        formatted.contains("information \"First\".\n        information \"Second\"."),
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
  before plan:
    assess <value>:
      0 <units> ... 10 <units>:
        information "OK".
"#;

    let formatted = format_script(input).expect("Formatter failed");
    let expected_line = "        information \"OK\".";
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
  before plan:
    information to <patient> "Hello":
      message expires after 1 day.
"#;

    let formatted = format_script(input).expect("Formatter failed");
    assert!(
        formatted.contains("information to <patient> \"Hello\":"),
        "Expected recipient in output, got:\n{}",
        formatted
    );

    parser::parse_plan(&formatted).expect("Formatted output should parse");
}

#[test]
fn formatter_round_trip_parse_format_parse() {
    let input = r#"
<point> is a unit:
    plural is <points>.

<best period> is a period:
    timeframe:
        between Monday ... Friday; 07:40 ... 07:50.

<severity> is a number:
    valid values:
        0 <points> ... 10 <points>.
    question:
        ask "How severe is it?".
    meaning of <severity>:
        valid meanings:
            <mild>; <moderate>; <severe>.
        assess meaning of <severity>:
            0 <points> ... 3 <points>:
                <mild>.
            4 <points> ... 7 <points>:
                <moderate>.
            8 <points> ... 10 <points>:
                <severe>.

<tracker> is a plan:
    before plan:
        ask <severity>.
        information "Tracking started.".
    every 1 day:
        ask <severity>.
        assess <severity>:
            0 <points> ... 3 <points>:
                information "Looking good.".
            4 <points> ... 10 <points>:
                information "Please monitor closely.".
"#;

    // First parse
    let plan1 = parser::parse_plan(input).expect("First parse failed");

    // Format
    let formatted = format_script(input).expect("Formatting failed");

    // Second parse from formatted output
    let plan2 = parser::parse_plan(&formatted).expect("Second parse (round-trip) failed");

    // Both should have the same number of definitions
    assert_eq!(
        plan1.definitions.len(),
        plan2.definitions.len(),
        "Round-trip changed definition count: {} vs {}",
        plan1.definitions.len(),
        plan2.definitions.len()
    );
}

#[test]
fn formatter_handles_all_definition_types() {
    let input = r#"
<mg> is a unit:
    plural is <mgs>.
    abbreviation is "mg".

<dose> is a unit:
    plural is <doses>.

<physician> is an addressee:
    contact information:
        email is "dr@hospital.com".

<morning> is a period:
    timeframe:
        between Monday ... Sunday; 08:00 ... 09:00.

<weight> is a number:
    unit is kg.
    valid values:
        1 kg ... 300 kg.
    question:
        ask "What is your weight?".

<status> is an enumeration:
    valid values:
        <stable>; <unstable>.

<aspirin> is a drug:
    ingredients:
        <acetylsalicylic acid> 500 mg.
    dosage safety:
        maximum single dose = 1000 mg.
        maximum daily dose = 4000 mg.

<care plan> is a plan:
    before plan:
        ask <weight>.
        information "Plan started.".
    every 1 day:
        information to <physician> "Daily report.".
"#;

    let formatted = format_script(input).expect("Formatting all definition types failed");
    parser::parse_plan(&formatted).expect("Formatted output with all definition types should parse");
}
