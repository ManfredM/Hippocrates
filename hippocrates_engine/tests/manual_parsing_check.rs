
mod fixture_loader;

use fixture_loader::{load_scenario, ScenarioKind};
use hippocrates_engine::parser;

#[test]
fn test_parse_treating_copd() {
    let input = load_scenario("tests/fixtures/runtime_plans.hipp", "copd_plan", ScenarioKind::Pass);
    match parser::parse_plan(&input) {
        Ok(_) => println!("treating_copd.hipp parsed successfully"),
        Err(e) => panic!("Failed to parse treating_copd.hipp: {:?}", e),
    }
}

#[test]
fn test_parse_99_bottles_v2() {
    let input = load_scenario("tests/fixtures/runtime_plans.hipp", "sing_plan", ScenarioKind::Pass);
    match parser::parse_plan(&input) {
        Ok(_) => println!("99_bottles_v2.hipp parsed successfully"),
        Err(e) => panic!("Failed to parse 99_bottles_v2.hipp: {:?}", e),
    }
}
