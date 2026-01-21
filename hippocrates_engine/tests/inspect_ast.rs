mod fixture_loader;

#[cfg(test)]
mod tests {
    use crate::fixture_loader::{load_scenario, ScenarioKind};
    use hippocrates_engine::ast::Definition;
    use hippocrates_engine::parser;

    #[test]
    fn inspect_copd_period_ast() {
        let content = load_scenario("tests/fixtures/runtime_plans.hipp", "copd_plan", ScenarioKind::Pass);
        let plan = parser::parse_plan(&content).expect("Failed to parse");

        for def in plan.definitions {
            match def {
                Definition::Value(v) => {
                    println!("Found Value: '{}' (Type: {:?})", v.name, v.value_type);
                    for prop in v.properties {
                        println!("  Prop: {:?}", prop);
                    }
                }
                Definition::Period(p) => {
                    println!("Found PeriodDef: '{}'", p.name);
                }
                Definition::Plan(p) => {
                    println!("Found Plan: '{}'", p.name);
                }
                _ => {
                    println!("Found Other: {:?}", def);
                }
            }
        }
    }
}
