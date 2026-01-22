// Non-spec integration/regression tests (ignored by default).

use crate::fixture_loader::{list_scenarios, load_scenario, ScenarioKind};
use hippocrates_engine::ast::Definition;
use hippocrates_engine::parser;
use hippocrates_engine::runtime::{validator, Engine, ExecutionMode};

#[test]
#[ignore = "Non-spec integration/regression"]
fn integration_plan_fixture_suite() {
    let path = "tests/fixtures/integration_plans.hipp";

    let pass_scenarios = list_scenarios(path, ScenarioKind::Pass);
    assert!(!pass_scenarios.is_empty(), "No PASS scenarios in {}", path);

    for name in pass_scenarios {
        let input = load_scenario(path, &name, ScenarioKind::Pass);
        let plan = parser::parse_plan(&input)
            .unwrap_or_else(|err| panic!("Failed to parse PASS scenario {}: {:?}", name, err));

        let result = validator::validate_file(&plan);
        assert!(
            result.is_ok(),
            "Validation failed for PASS scenario {}: {:?}",
            name,
            result.err()
        );

        let plan_names: Vec<String> = plan
            .definitions
            .iter()
            .filter_map(|def| match def {
                Definition::Plan(plan_def) => Some(plan_def.name.clone()),
                _ => None,
            })
            .collect();

        assert!(
            !plan_names.is_empty(),
            "No plans found in PASS scenario {}",
            name
        );

        let mut engine = Engine::new();
        engine.set_mode(ExecutionMode::Simulation {
            speed_factor: None,
            duration: Some(chrono::Duration::hours(1)),
        });
        engine.load_plan(plan);

        for plan_name in plan_names {
            engine.execute(&plan_name);
        }
    }

    let fail_scenarios = list_scenarios(path, ScenarioKind::Fail);
    assert!(!fail_scenarios.is_empty(), "No FAIL scenarios in {}", path);

    for name in fail_scenarios {
        let input = load_scenario(path, &name, ScenarioKind::Fail);
        let plan = parser::parse_plan(&input)
            .unwrap_or_else(|err| panic!("Failed to parse FAIL scenario {}: {:?}", name, err));

        let result = validator::validate_file(&plan);
        assert!(
            result.is_err(),
            "Validation unexpectedly passed for FAIL scenario {}",
            name
        );
    }
}
