// Spec §3.4, §4.2: value definitions and properties.

use crate::fixture_loader::{load_scenario, ScenarioKind};
use hippocrates_engine::ast::{Definition, Property};
use hippocrates_engine::domain::ValueType;
use hippocrates_engine::parser;

// REQ-3.4-01: value definitions parse from fixtures.
#[test]
fn spec_value_definition_parsing() {
    let input = load_scenario("tests/fixtures/specs.hipp", "value_definition", ScenarioKind::Pass);

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

    let mut unit_found = false;
    let mut bp_found = false;
    let mut status_found = false;

    for def in plan.definitions {
        match def {
            Definition::Unit(ud) => {
                if ud.name == "TestUnit" {
                    unit_found = true;
                    assert!(ud.plurals.contains(&"TestUnits".to_string()) || ud.plurals.contains(&"<TestUnits>".to_string()));
                    assert!(ud.abbreviations.contains(&"tu".to_string()));
                }
            }
            Definition::Value(vd) => {
                if vd.name == "BloodPressure" {
                    bp_found = true;
                    assert!(matches!(vd.value_type, ValueType::Number));
                } else if vd.name == "Status" {
                    status_found = true;
                    assert!(matches!(vd.value_type, ValueType::Enumeration));
                }
            }
            _ => {}
        }
    }

    assert!(unit_found, "Unit definition not found");
    assert!(bp_found, "BloodPressure definition not found");
    assert!(status_found, "Status definition not found");
}

// REQ-3.4-02: value type variants parse correctly.
#[test]
fn spec_value_type_variants_parse() {
    let input = r#"
<free text> is a string.
<when> is a time indication.
<timestamp> is a date/time.
<period ref> is a period.
<plan ref> is a plan.
<drug ref> is a drug.
<addressee ref> is an addressee.
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");

    let mut seen = std::collections::HashMap::new();
    for def in plan.definitions {
        if let Definition::Value(v) = def {
            seen.insert(v.name.clone(), v.value_type.clone());
        }
    }

    assert!(matches!(seen.get("free text"), Some(ValueType::String)));
    assert!(matches!(seen.get("when"), Some(ValueType::TimeIndication)));
    assert!(matches!(seen.get("timestamp"), Some(ValueType::DateTime)));
    assert!(matches!(seen.get("period ref"), Some(ValueType::Period)));
    assert!(matches!(seen.get("plan ref"), Some(ValueType::Plan)));
    assert!(matches!(seen.get("drug ref"), Some(ValueType::Drug)));
    assert!(matches!(seen.get("addressee ref"), Some(ValueType::Addressee)));
}

// REQ-3.4-03: unit properties parse in numeric values.
#[test]
fn spec_unit_property_parsing() {
    let input = r#"
<weight> is a number:
    unit is kg.
    valid values:
        0 kg ... 10 kg.

<mass> is a number:
    unit: g.
    valid values:
        0 g ... 100 g.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");

    let mut weight_unit = None;
    let mut mass_unit = None;

    for def in plan.definitions {
        if let Definition::Value(v) = def {
            if v.name == "weight" {
                weight_unit = v.properties.iter().find_map(|p| {
                    if let Property::Unit(unit) = p { Some(unit.clone()) } else { None }
                });
            } else if v.name == "mass" {
                mass_unit = v.properties.iter().find_map(|p| {
                    if let Property::Unit(unit) = p { Some(unit.clone()) } else { None }
                });
            }
        }
    }

    assert!(weight_unit.is_some(), "Expected unit is kg to parse");
    assert!(mass_unit.is_some(), "Expected unit: g to parse");
}

// REQ-3.4-04: value timeframe properties parse.
#[test]
fn spec_value_timeframe_property_parsing() {
    let input = r#"
<windowed> is a number:
    valid values:
        0 kg ... 10 kg.
    timeframe:
        between Monday ... Sunday; 08:00 ... 18:00.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut timeframe_found = false;

    for def in plan.definitions {
        if let Definition::Value(v) = def {
            if v.name == "windowed" {
                timeframe_found = v.properties.iter().any(|p| matches!(p, Property::Timeframe(_)));
            }
        }
    }

    assert!(timeframe_found, "Expected timeframe property to parse");
}

// REQ-3.4-05: inheritance properties parse with overrides.
#[test]
fn spec_inheritance_property_parsing() {
    let input = r#"
<Base> is a number:
    valid values:
        0 kg ... 10 kg.

<Child> is a number:
    definition is the same as for <Base> except:
        documentation:
            english: "Overrides".
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut inheritance_found = false;

    for def in plan.definitions {
        if let Definition::Value(v) = def {
            if v.name == "Child" {
                inheritance_found = v.properties.iter().any(|p| matches!(p, Property::Inheritance(base, overrides) if base == "Base" && overrides.is_some()));
            }
        }
    }

    assert!(inheritance_found, "Expected inheritance property with overrides");
}

// REQ-3.4-06: documentation properties parse in inline and block forms.
#[test]
fn spec_documentation_property_parsing() {
    let input = r#"
<inline doc> is a string:
    documentation:
        english: "Inline doc".

<block doc> is a string:
    documentation:
        english:
            "Block doc".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut inline_ok = false;
    let mut block_ok = false;

    for def in plan.definitions {
        if let Definition::Value(v) = def {
            if v.name == "inline doc" {
                inline_ok = v.properties.iter().any(|p| matches!(p, Property::Documentation(text) if text == "Inline doc"));
            } else if v.name == "block doc" {
                block_ok = v.properties.iter().any(|p| matches!(p, Property::Documentation(text) if text == "Block doc"));
            }
        }
    }

    assert!(inline_ok, "Inline documentation should parse");
    assert!(block_ok, "Block documentation should parse");
}

// REQ-3.4-07: custom properties parse as generic properties.
#[test]
fn spec_generic_property_parsing() {
    let input = r#"
<note> is a string:
    <custom property>:
        Custom note.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut custom_found = false;

    for def in plan.definitions {
        if let Definition::Value(v) = def {
            if v.name == "note" {
                custom_found = v.properties.iter().any(|p| matches!(p, Property::Custom(name, content) if name == "custom property" && content.contains("Custom note")));
            }
        }
    }

    assert!(custom_found, "Expected custom property to parse");
}
