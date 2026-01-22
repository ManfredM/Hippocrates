use hippocrates_engine::runtime::scheduler::Scheduler;
use hippocrates_engine::parser;
use hippocrates_engine::ast::Definition;
use chrono::{Utc, TimeZone};

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_scheduler_copd_logic() {
    let input = r#"
<best period> is a period:
    timeframe:
        between Monday ... Friday; 07:40 ... 07:50.

"#;

    let plan_struct = parser::parse_plan(input).expect("Failed to parse");
    
    // Simulate Environment loading
    let mut defs = std::collections::HashMap::new();
    for def in plan_struct.definitions {
        match def {
            Definition::Value(v) => {
                defs.insert(v.name.clone(), Definition::Value(v));
            }
            Definition::Plan(p) => {
                 defs.insert(p.name.clone(), Definition::Plan(p));
            }
            Definition::Period(p) => {
                defs.insert(p.name.clone(), Definition::Period(p));
            }
            _ => {}
        }
    }
    
    // Debug keys
    println!("Keys: {:?}", defs.keys());
    
    // Verify lookup
    let target_name = "best period"; // Based on usage in trigger
    assert!(defs.contains_key(target_name), "Should contain key 'best period'");

    let def = defs.get(target_name).expect("Def must exist");
    
    // Scheduler test
    let now = Utc.with_ymd_and_hms(2026, 1, 18, 12, 0, 0).unwrap().naive_utc();
    let next = Scheduler::next_occurrence(def, now);
    assert!(next.is_some());
    println!("Next: {:?}", next.unwrap());
}
