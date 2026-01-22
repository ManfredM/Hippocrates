// Spec §3.9, §3.10: addressees and drug definitions.

use crate::fixture_loader::{load_scenario, ScenarioKind};
use hippocrates_engine::ast::{AdminRule, Definition, DosageRule, Expression, Literal, Property};
use hippocrates_engine::parser;
use hippocrates_engine::runtime::validator;

// REQ-3.10-01: drug definition validation rejects undefined units.
#[test]
fn spec_drug_definition_validation() {
    let input = load_scenario("tests/fixtures/specs.hipp", "drug_definition", ScenarioKind::Fail);

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");
    let result = validator::validate_file(&plan);

    assert!(result.is_err(), "Validation passed but should fail due to undefined unit <BadUnit>");

    let errors = result.err().unwrap();
    assert!(errors.iter().any(|e| e.message.contains("Undefined unit")), "Errors should contain 'Undefined unit': {:?}", errors);
}

// REQ-3.9-01: addressee groups and contact logic parse.
#[test]
fn spec_addressee_group_and_contact_logic_parsing() {
    let input = load_scenario("tests/fixtures/specs.hipp", "full_spec", ScenarioKind::Pass);
    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

    let addressee = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Addressee(a) = d { Some(a) } else { None })
        .expect("Addressee not found");

    assert_eq!(addressee.name, "Dr. House");

    let group = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Addressee(a) = d { if a.name == "OncologyTeam" { Some(a) } else { None } } else { None })
        .expect("Addressee group not found");

    assert!(group.is_group, "Expected OncologyTeam to be an addressee group");

    let grouped = group.properties.iter().find_map(|p| {
        if let Property::GroupedAddressees(items) = p { Some(items.clone()) } else { None }
    });
    assert!(grouped.as_ref().map(|g| g.contains(&"Dr. House".to_string())).unwrap_or(false));
    assert!(grouped.as_ref().map(|g| g.contains(&"Dr. Wilson".to_string())).unwrap_or(false));

    let contact_order = group.properties.iter().find_map(|p| {
        if let Property::ContactOrder(order) = p { Some(order.clone()) } else { None }
    });
    assert_eq!(contact_order.as_deref(), Some("Parallel"));

    let after_consent = addressee
        .properties
        .iter()
        .any(|p| matches!(p, Property::AfterConsentRejected(_)));
    assert!(after_consent, "Expected after consent block for Dr. House");
}

// REQ-3.10-02: drug interaction properties parse.
#[test]
fn spec_drug_interactions_parse() {
    let input = load_scenario("tests/fixtures/specs.hipp", "full_spec", ScenarioKind::Pass);
    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

    let drug = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Drug(drug) = d { Some(drug) } else { None })
        .expect("Drug definition not found");

    let has_interactions = drug
        .properties
        .iter()
        .any(|p| matches!(p, Property::Interactions(_)));

    assert!(has_interactions, "Expected interactions property on drug");
}

// REQ-3.9-02: contact details and sequence contact order parse.
#[test]
fn spec_addressee_contact_details_and_sequence_order_parsing() {
    let input = r#"
<Dr. A> is an addressee:
    contact information:
        email is "a@example.com".
        phone is "+1-555-0100".
        hippocrates id is "A-1".

<Care Team> is an addressee group:
    grouped addressees are <Dr. A>.
    contact information:
        email is "team@example.com".
        phone is "+1-555-0200".
        hippocrates id is "TEAM-1".
    order of contacting:
        sequence of contacting is <Dr. A>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");

    let group = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Addressee(a) = d { if a.name == "Care Team" { Some(a) } else { None } } else { None })
        .expect("Addressee group not found");

    let contact_info = group.properties.iter().find_map(|p| {
        if let Property::ContactInfo(details) = p { Some(details) } else { None }
    }).expect("Contact info not found");

    assert!(contact_info.iter().any(|d| matches!(d, hippocrates_engine::ast::ContactDetail::Email(_))));
    assert!(contact_info.iter().any(|d| matches!(d, hippocrates_engine::ast::ContactDetail::Phone(_))));
    assert!(contact_info.iter().any(|d| matches!(d, hippocrates_engine::ast::ContactDetail::HippocratesId(_))));

    let contact_order = group.properties.iter().find_map(|p| {
        if let Property::ContactOrder(order) = p { Some(order.as_str()) } else { None }
    });
    assert_eq!(contact_order, Some("Sequence"));
}

// REQ-3.10-03: dosage safety and administration rules parse.
#[test]
fn spec_drug_dosage_and_admin_rules_parsing() {
    let input = r#"
<Aspirin> is a drug:
    dosage safety:
        maximum single dose = 2 mg.
        maximum daily dose = 5 mg.
        minimum time between doses = 6 hours.
    administration:
        form of administration is <tablet>.
        <Aspirin> 2 mg after <meal>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");

    let drug = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Drug(drug) = d { Some(drug) } else { None })
        .expect("Drug definition not found");

    let dosage_rules = drug.properties.iter().find_map(|p| {
        if let Property::DosageSafety(rules) = p { Some(rules) } else { None }
    }).expect("Dosage safety not found");

    assert!(dosage_rules.iter().any(|r| matches!(r, DosageRule::MaxSingle(_))));
    assert!(dosage_rules.iter().any(|r| matches!(r, DosageRule::MaxDaily(_))));
    assert!(dosage_rules.iter().any(|r| matches!(r, DosageRule::MinTimeBetween(_))));

    let admin_rules = drug.properties.iter().find_map(|p| {
        if let Property::Administration(rules) = p { Some(rules) } else { None }
    }).expect("Administration rules not found");

    assert!(admin_rules.iter().any(|r| matches!(r, AdminRule::Form(name) if name == "tablet")));

    let mut saw_schedule = false;
    for rule in admin_rules {
        if let AdminRule::Schedule(drug_name, expr, after) = rule {
            if drug_name == "Aspirin" && after == "meal" {
                if let Expression::Literal(Literal::Quantity(value, unit, _)) = expr {
                    assert_eq!(*value, 2.0);
                    assert!(matches!(unit, hippocrates_engine::domain::Unit::Milligram));
                    saw_schedule = true;
                }
            }
        }
    }
    assert!(saw_schedule, "Expected administration schedule rule to parse");
}
