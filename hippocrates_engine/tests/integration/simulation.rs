use hippocrates_engine::ast::{PeriodDef, RangeSelector, Expression, Literal, Definition};
use hippocrates_engine::runtime::scheduler::Scheduler;
use chrono::{TimeZone, Utc, Duration};

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_occurrences_simulation_logic() {
    // ... setup ...
    let period_def = PeriodDef {
        name: "Morning".to_string(),
        line: 0,
        timeframes: vec![vec![
            RangeSelector::Equals(Expression::Literal(Literal::TimeOfDay("08:00".to_string())))
        ]],
    };
    
    // Start Monday 2026-01-01 (Thursday)
    let start_time = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap().naive_utc();
    let end_time = start_time + Duration::days(3); // 3 days: 1st, 2nd, 3rd.
    
    let mut occurrences = Vec::new();
    let mut current_time = start_time;
    
    // Loop logic from FFI
    while current_time < end_time {
        if let Some((start, _)) = Scheduler::next_occurrence(&Definition::Period(period_def.clone()), current_time) {
            if start >= end_time {
                break;
            }
            occurrences.push(start);
            current_time = start + Duration::seconds(1);
        } else {
            break;
        }
    }
    
    assert_eq!(occurrences.len(), 3);
    assert_eq!(occurrences[0], Utc.with_ymd_and_hms(2026, 1, 1, 8, 0, 0).unwrap().naive_utc());
    assert_eq!(occurrences[1], Utc.with_ymd_and_hms(2026, 1, 2, 8, 0, 0).unwrap().naive_utc());
    assert_eq!(occurrences[2], Utc.with_ymd_and_hms(2026, 1, 3, 8, 0, 0).unwrap().naive_utc());
}
