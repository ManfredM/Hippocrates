#[test]
#[ignore = "Non-spec integration/regression"]
fn test_scheduler_duplication() {
    use hippocrates_engine::ast::Definition;
    use hippocrates_engine::runtime::scheduler::Scheduler;
    use chrono::{TimeZone, Utc, Datelike, Timelike};

    let src = r#"
<best inhalation period> is a period:
    timeframe:
        between Monday ... Friday; 07:40 ... 07:50.
        between Saturday ... Sunday; 09:00 ... 09:10.
"#;
    let plan = hippocrates_engine::parser::parse_plan(src).expect("Parse failed");
    
    let def = plan.definitions.iter().find(|d| {
        match d {
            Definition::Period(p) => p.name == "best inhalation period",
            _ => false,
        }
    }).expect("Def not found");
    
    println!("DEBUG: Def Structure: {:?}", def); // IMPORTANT: Print structure

    // Test Date: Monday, Jan 19, 2026 (Mon)
    let base_date = Utc.with_ymd_and_hms(2026, 1, 19, 6, 0, 0).unwrap().naive_utc(); 
    
    println!("Base date: {}", base_date);

    // Expected: 07:40
    if let Some((start, _)) = Scheduler::next_occurrence(def, base_date) {
        println!("Next occurrence: {}", start);
        assert_eq!(start.hour(), 7);
        assert_eq!(start.minute(), 40);
        assert_eq!(start.day(), 19);
    } else {
        panic!("Should have found occurrence");
    }

    // Now advance past 07:40+duration to see if next one is tomorrow (Tue 07:40) OR incorrectly today later (Mon 09:00)
    let after_first = Utc.with_ymd_and_hms(2026, 1, 19, 8, 0, 0).unwrap().naive_utc();
    if let Some((start, _)) = Scheduler::next_occurrence(def, after_first) {
        println!("Next occurrence after first: {}", start);
        if start.day() == 19 {
             println!("FAILURE: Found another occurrence on Monday at {}", start);
             assert!(false, "Should not find another occurrence on Monday");
        }
        assert_eq!(start.day(), 20); // Tuesday
        assert_eq!(start.hour(), 7);
        assert_eq!(start.minute(), 40);
    }
}
