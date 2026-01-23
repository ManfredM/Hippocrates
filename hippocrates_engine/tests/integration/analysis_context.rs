use hippocrates_engine::ast::{Definition, PlanBlock};
use hippocrates_engine::runtime::{Environment, Executor};
use std::sync::{Arc, atomic::AtomicBool, Mutex};

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_analysis_context_execution() {
    // Simplified test without context_for_analysis nesting
    let source = "\
<testunit> is a unit:
    plural is <testunits>.

<status> is an enumeration:
    valid values:
        <low>; <high>.

<test var> is a number:
    valid values:
        0 <testunits> ... 100 <testunits>.

<period> is a period:
    timeframe:
        between Monday ... Sunday; 00:00 ... 23:59.

<test plan> is a plan:
    begin of <period>:
        assess <test var>:
            0 <testunits> ... 10 <testunits>:
                <status> = <low>.
            11 <testunits> ... 100 <testunits>:
                <status> = <high>.
";

    // 1. Parsing Check with assignment
    let mut env = Environment::new();
    let plan = hippocrates_engine::parser::parse_plan(source).expect("Failed to parse plan");
    env.load_plan(plan);

    let stop_signal = Arc::new(AtomicBool::new(false));
    let _executor = Executor::new(stop_signal.clone());

    // 2. ShowMessage Test - simplified without context
    let source_stat = "\
<valunit> is a unit:
    plural is <valunits>.

<status> is an enumeration:
    valid values:
        <zero>.

<flag> is an enumeration:
    valid values:
        <Yes>; <No>.

<val> is a number:
    valid values:
        0 <valunits> ... 100 <valunits>.

<p> is a period:
    timeframe:
        between Monday ... Sunday; 00:00 ... 23:59.

<stat plan> is a plan:
    begin of <p>:
        timeframe for analysis is between 5 days ago ... now:
            <val> = count of <flag> is <Yes>.
        assess <val>:
            Not enough data:
                show message \"We do not have enough data\".
            0 <valunits> ... 100 <valunits>:
                <status> = <zero>.
";
    
    let plan_stat = hippocrates_engine::parser::parse_plan(source_stat).expect("Failed to parse stat plan");
    let mut env_stat = Environment::new();
    env_stat.load_plan(plan_stat);
    
    // Set start time
    let now = chrono::Utc::now().naive_utc();
    env_stat.set_start_time(now);
    env_stat.set_time(now);
    
    // Capture logs
    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = logs.clone();
    
    // Re-create executor with callback
    let mut executor_logging = Executor::with_activites(
            Box::new(|_| {}), 
            Box::new(move |msg, _, _| {
                logs_clone.lock().unwrap().push(msg);
            })
        );

    // Extract block
    let def = env_stat.definitions.get("stat plan").unwrap();
    let plan_def = match def { Definition::Plan(p) => p, _ => panic!("Not a plan") };
    let block = match &plan_def.blocks[0] { 
        PlanBlock::Trigger(tb) => tb.statements.clone(),
        _ => panic!("Not a trigger block") 
    };
    
    executor_logging.execute_block(&mut env_stat, &block);
    
    // Check Result
    let captured = logs.lock().unwrap();
    println!("Captured logs: {:?}", captured);
    assert!(captured.contains(&"We do not have enough data".to_string()));
}
