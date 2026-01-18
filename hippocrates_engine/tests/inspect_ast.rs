#[cfg(test)]
mod tests {
    use hippocrates_engine::ast::{Definition, Expression, Literal, RangeSelector};
    use hippocrates_engine::parser;
    use std::fs;

    #[test]
    fn inspect_copd_period_ast() {
        let content = fs::read_to_string("plans/treating_copd.hipp").expect("Failed to read plan");
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
