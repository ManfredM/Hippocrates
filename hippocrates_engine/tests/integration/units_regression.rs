// Non-spec integration/regression tests (ignored by default).

use hippocrates_engine::domain::RuntimeValue;
use hippocrates_engine::parser;
use hippocrates_engine::runtime::executor::Executor;
use hippocrates_engine::runtime::environment::Environment;

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_strict_units_without_definition() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 100 kg.

<plan> is a plan:
    during plan:
        <val> = 5 <coins> + 1 <coin>.
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");

    let mut env = Environment::new();
    env.load_plan(plan);

    let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut exe = Executor::new(stop_signal);

    exe.execute_plan(&mut env, "plan");

    let val = env.get_value("val").expect("Val not set");
    match val {
        RuntimeValue::Number(n) => {
            assert_eq!(*n, 6.0);
        }
        RuntimeValue::Quantity(_, u) => {
            panic!("Should have mismatched units! Got matching unit: {:?}", u);
        }
        _ => panic!("Unexpected result {:?}", val),
    }
}
