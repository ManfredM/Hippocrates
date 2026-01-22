use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs;
use std::io::Write;
use std::process;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use hippocrates_engine::ast::Definition;
use hippocrates_engine::domain::{InputMessage, RuntimeValue, Unit, ValueType};
use hippocrates_engine::format_script;
use hippocrates_engine::parser::parse_plan;
use hippocrates_engine::runtime::{Environment, ExecutionMode, Executor, format_identifier, normalize_identifier};
use serde_json::Value;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "simulate" => run_simulate(&args[2..]),
        "format" => run_format(&args[2..]),
        "--help" | "-h" => {
            print_usage();
            process::exit(0);
        }
        _ => run_validate(&args[1]),
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  hippocrates-cli <file_path>");
    eprintln!(
        "  hippocrates-cli simulate <file_path> [--plan <plan_name>] [--answers <answers.json>] [--mode real-time|time-lapse] [--duration <duration>]"
    );
    eprintln!("  hippocrates-cli format <file_path> [--write]");
}

fn print_simulate_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  hippocrates-cli simulate <file_path> [--plan <plan_name>] [--answers <answers.json>] [--mode real-time|time-lapse] [--duration <duration>]"
    );
}

fn print_format_usage() {
    eprintln!("Usage:");
    eprintln!("  hippocrates-cli format <file_path> [--write]");
}

fn run_validate(file_path: &str) {
    let content = read_file_or_exit(file_path);

    match parse_plan(&content) {
        Ok(plan) => match hippocrates_engine::runtime::validator::validate_file(&plan) {
            Ok(_) => {
                println!("{{ \"status\": \"valid\" }}");
                process::exit(0);
            }
            Err(errors) => {
                match serde_json::to_string_pretty(&errors) {
                    Ok(json) => println!("{}", json),
                    Err(err) => eprintln!("Error serializing errors: {}", err),
                }
                process::exit(1);
            }
        },
        Err(e) => {
            match serde_json::to_string_pretty(&e) {
                Ok(json) => println!("{}", json),
                Err(err) => eprintln!("Error serializing error: {}", err),
            }
            process::exit(1);
        }
    }
}

fn run_simulate(args: &[String]) {
    if args.is_empty() {
        print_simulate_usage();
        process::exit(1);
    }

    let mut file_path: Option<String> = None;
    let mut plan_name: Option<String> = None;
    let mut answers_path: Option<String> = None;

    let mut mode = "time-lapse".to_string();
    let mut duration_arg: Option<String> = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--plan" => {
                plan_name = Some(next_arg_or_exit(&mut iter, "--plan"));
            }
            "--answers" => {
                answers_path = Some(next_arg_or_exit(&mut iter, "--answers"));
            }
            "--mode" => {
                mode = next_arg_or_exit(&mut iter, "--mode");
            }
            "--duration" => {
                duration_arg = Some(next_arg_or_exit(&mut iter, "--duration"));
            }
            "--help" | "-h" => {
                print_simulate_usage();
                process::exit(0);
            }
            _ => {
                if file_path.is_none() {
                    file_path = Some(arg.clone());
                } else {
                    eprintln!("Unexpected argument: {}", arg);
                    print_simulate_usage();
                    process::exit(1);
                }
            }
        }
    }

    let file_path = match file_path {
        Some(p) => p,
        None => {
            print_simulate_usage();
            process::exit(1);
        }
    };

    let content = read_file_or_exit(&file_path);
    let plan = match parse_plan(&content) {
        Ok(plan) => plan,
        Err(e) => {
            if let Ok(json) = serde_json::to_string_pretty(&e) {
                println!("{}", json);
            } else {
                eprintln!("Parse error: {}", e);
            }
            process::exit(1);
        }
    };

    if let Err(errors) = hippocrates_engine::runtime::validator::validate_file(&plan) {
        match serde_json::to_string_pretty(&errors) {
            Ok(json) => println!("{}", json),
            Err(err) => eprintln!("Error serializing errors: {}", err),
        }
        process::exit(1);
    }

    let plan_name = match plan_name {
        Some(name) => name,
        None => select_single_plan_name(&plan),
    };

    let value_types = Arc::new(build_value_type_map(&plan));
    let output_lock = Arc::new(Mutex::new(()));
    let mut env = Environment::new();
    env.load_plan(plan);
    let output_lock_clone = Arc::clone(&output_lock);
    env.set_output_handler(Arc::new(move |msg| {
        write_stdout_line(&output_lock_clone, &msg);
    }));

    let unit_map = Arc::new(env.unit_map.clone());
    let answers = match answers_path.as_ref() {
        Some(path) => match load_answers(path) {
            Ok(map) => map,
            Err(err) => {
                eprintln!("Failed to load answers: {}", err);
                process::exit(1);
            }
        },
        None => HashMap::new(),
    };

    let answers = Arc::new(Mutex::new(answers));
    let has_answers = answers_path.is_some();

    let duration = match duration_arg {
        Some(value) => match parse_duration_arg(&value) {
            Ok(duration) => Some(duration),
            Err(err) => {
                eprintln!("Invalid duration '{}': {}", value, err);
                process::exit(1);
            }
        },
        None => None,
    };

    let (tx_input, rx_input) = std::sync::mpsc::channel();
    let mut executor = Executor::new(Arc::new(AtomicBool::new(false)));
    match mode.as_str() {
        "real-time" => executor.set_mode(ExecutionMode::RealTime),
        "time-lapse" => executor.set_mode(ExecutionMode::Simulation {
            speed_factor: None,
            duration,
        }),
        _ => {
            eprintln!("Invalid mode '{}'. Use 'real-time' or 'time-lapse'.", mode);
            process::exit(1);
        }
    }

    if duration.is_some() && mode == "real-time" {
        eprintln!("--duration is only supported with --mode time-lapse.");
        process::exit(1);
    }
    executor.set_input_receiver(rx_input);

    let is_time_lapse = mode == "time-lapse";
    let answers_clone = Arc::clone(&answers);
    let unit_map_clone = Arc::clone(&unit_map);
    let value_types_clone = Arc::clone(&value_types);
    let tx_input_clone = tx_input.clone();
    let output_lock_clone = Arc::clone(&output_lock);
    executor.set_ask_callback(Box::new(move |req| {
        if let Ok(json) = serde_json::to_string(&req) {
            write_stdout_line(&output_lock_clone, &json);
        }

        if !has_answers {
            eprintln!(
                "No answers file provided; cannot answer question for '{}'.",
                req.variable_name
            );
            process::exit(1);
        }

        let answer_key = format_identifier(&req.variable_name);
        let normalized_name = normalize_identifier(&req.variable_name);

        let next_answer = {
            let mut guard = answers_clone.lock().unwrap();
            guard.get_mut(&answer_key).and_then(|queue| queue.pop_front())
        };

        let answer_entry = match next_answer {
            Some(entry) => entry,
            None => {
                eprintln!("No answer available for '{}'.", req.variable_name);
                process::exit(1);
            }
        };

        let value_type = match value_types_clone.get(&normalized_name) {
            Some(vt) => vt,
            None => {
                eprintln!("No value definition found for '{}'.", req.variable_name);
                process::exit(1);
            }
        };

        let runtime_value = match parse_answer_value(value_type, &answer_entry.value, &unit_map_clone) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("Invalid answer for '{}': {}", req.variable_name, err);
                process::exit(1);
            }
        };

        let mut timestamp = DateTime::<Utc>::from_timestamp_millis(req.timestamp)
            .unwrap_or_else(Utc::now)
            .naive_utc();
        if let Some(delay) = answer_entry.delay {
            if is_time_lapse {
                timestamp = timestamp + delay;
            }
        }

        let msg = InputMessage {
            variable: normalized_name,
            value: runtime_value,
            timestamp,
        };

        if let Err(err) = tx_input_clone.send(msg) {
            eprintln!(
                "Failed to send answer for '{}': {}",
                req.variable_name, err
            );
            process::exit(1);
        }
    }));

    if !env.definitions.contains_key(&plan_name) {
        eprintln!("Plan '{}' not found in plan definitions.", plan_name);
        process::exit(1);
    }

    executor.execute_plan(&mut env, &plan_name);
    write_stdout_line(
        &output_lock,
        &format!(
            "[{}] Simulation finished.",
            env.now.format("%Y-%m-%d %H:%M:%S")
        ),
    );
}

fn run_format(args: &[String]) {
    if args.is_empty() {
        print_format_usage();
        process::exit(1);
    }

    let mut file_path: Option<String> = None;
    let mut write_in_place = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--write" => write_in_place = true,
            "--help" | "-h" => {
                print_format_usage();
                process::exit(0);
            }
            _ => {
                if file_path.is_none() {
                    file_path = Some(arg.clone());
                } else {
                    eprintln!("Unexpected argument: {}", arg);
                    print_format_usage();
                    process::exit(1);
                }
            }
        }
    }

    let file_path = match file_path {
        Some(p) => p,
        None => {
            print_format_usage();
            process::exit(1);
        }
    };

    let content = read_file_or_exit(&file_path);
    match format_script(&content) {
        Ok(formatted) => {
            if write_in_place {
                if let Err(err) = fs::write(&file_path, formatted) {
                    eprintln!("Error writing file: {}", err);
                    process::exit(1);
                }
            } else {
                print!("{}", formatted);
            }
            process::exit(0);
        }
        Err(err) => {
            match serde_json::to_string_pretty(&err) {
                Ok(json) => println!("{}", json),
                Err(e) => eprintln!("Error serializing error: {}", e),
            }
            process::exit(1);
        }
    }
}

fn read_file_or_exit(file_path: &str) -> String {
    match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            process::exit(1);
        }
    }
}

fn write_stdout_line(lock: &Arc<Mutex<()>>, line: &str) {
    let _guard = lock.lock().unwrap();
    let mut stdout = std::io::stdout();
    let _ = writeln!(stdout, "{}", line);
    let _ = stdout.flush();
}

fn next_arg_or_exit<'a, I>(iter: &mut I, flag: &str) -> String
where
    I: Iterator<Item = &'a String>,
{
    match iter.next() {
        Some(value) => value.clone(),
        None => {
            eprintln!("Missing value for {}", flag);
            process::exit(1);
        }
    }
}

fn select_single_plan_name(plan: &hippocrates_engine::ast::Plan) -> String {
    let plan_names: Vec<String> = plan
        .definitions
        .iter()
        .filter_map(|def| match def {
            Definition::Plan(plan_def) => Some(plan_def.name.clone()),
            _ => None,
        })
        .collect();

    if plan_names.len() == 1 {
        return plan_names[0].clone();
    }

    if plan_names.is_empty() {
        eprintln!("No plan definitions found.");
        process::exit(1);
    }

    eprintln!(
        "Multiple plans found ({}). Specify one with --plan.",
        plan_names.len()
    );
    for name in plan_names {
        eprintln!("  {}", name);
    }
    process::exit(1);
}

fn build_value_type_map(plan: &hippocrates_engine::ast::Plan) -> HashMap<String, ValueType> {
    let mut map = HashMap::new();
    for def in &plan.definitions {
        if let Definition::Value(value_def) = def {
            map.insert(value_def.name.clone(), value_def.value_type.clone());
        }
    }
    map
}

#[derive(Debug, Clone)]
struct AnswerEntry {
    value: Value,
    delay: Option<chrono::Duration>,
}

type AnswerMap = HashMap<String, VecDeque<AnswerEntry>>;

fn load_answers(path: &str) -> Result<AnswerMap, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Error reading answers file: {}", e))?;
    let json: Value =
        serde_json::from_str(&content).map_err(|e| format!("Error parsing answers JSON: {}", e))?;

    let mut map: AnswerMap = HashMap::new();

    match json {
        Value::Object(obj) => {
            for (key, value) in obj {
                let name = format_identifier(&key);
                if let Value::Array(items) = value {
                    for item in items {
                        push_answer(&mut map, &name, item)?;
                    }
                } else {
                    push_answer(&mut map, &name, value)?;
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                let obj = item
                    .as_object()
                    .ok_or_else(|| "Answer list items must be objects".to_string())?;
                let variable = obj
                    .get("variable")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        "Answer items must include a string 'variable' field".to_string()
                    })?;
                let value = obj
                    .get("value")
                    .cloned()
                    .ok_or_else(|| "Answer items must include a 'value' field".to_string())?;
                let delay = obj.get("delay").cloned();
                let name = format_identifier(variable);
                push_answer_with_delay(&mut map, &name, value, delay)?;
            }
        }
        _ => {
            return Err("Answers file must be a JSON object or array.".to_string());
        }
    }

    Ok(map)
}

fn push_answer(map: &mut AnswerMap, variable: &str, value: Value) -> Result<(), String> {
    let entry = parse_answer_entry(value)?;
    map.entry(variable.to_string())
        .or_insert_with(VecDeque::new)
        .push_back(entry);
    Ok(())
}

fn push_answer_with_delay(
    map: &mut AnswerMap,
    variable: &str,
    value: Value,
    delay: Option<Value>,
) -> Result<(), String> {
    let mut entry = parse_answer_entry(value)?;
    if let Some(delay_value) = delay {
        let delay = parse_delay_value(&delay_value)?;
        if entry.delay.is_some() {
            return Err("Delay specified multiple times for an answer.".to_string());
        }
        entry.delay = Some(delay);
    }
    map.entry(variable.to_string())
        .or_insert_with(VecDeque::new)
        .push_back(entry);
    Ok(())
}

fn parse_answer_value(
    value_type: &ValueType,
    value: &Value,
    unit_map: &HashMap<String, Unit>,
) -> Result<RuntimeValue, String> {
    match value_type {
        ValueType::Number => parse_quantity_value(value, unit_map),
        ValueType::DateTime => {
            let text = parse_string_value(value)?;
            if let Some(dt) = parse_date_time_literal(&text) {
                Ok(RuntimeValue::Date(dt))
            } else if parse_time_literal(&text).is_some() {
                Ok(RuntimeValue::String(text))
            } else {
                Err(format!("Invalid date/time literal '{}'", text))
            }
        }
        ValueType::Enumeration => {
            let text = parse_string_value(value)?;
            Ok(RuntimeValue::Enumeration(text))
        }
        ValueType::String
        | ValueType::TimeIndication
        | ValueType::Period
        | ValueType::Plan
        | ValueType::Drug
        | ValueType::Addressee
        | ValueType::AddresseeGroup => {
            let text = parse_string_value(value)?;
            Ok(RuntimeValue::String(text))
        }
    }
}

fn parse_date_time_literal(value: &str) -> Option<NaiveDateTime> {
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M") {
        return Some(dt);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %-H:%M") {
        return Some(dt);
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return date.and_hms_opt(0, 0, 0);
    }
    None
}

fn parse_time_literal(value: &str) -> Option<()> {
    if NaiveTime::parse_from_str(value, "%H:%M").is_ok()
        || NaiveTime::parse_from_str(value, "%-H:%M").is_ok()
    {
        return Some(());
    }
    None
}

fn parse_answer_entry(value: Value) -> Result<AnswerEntry, String> {
    match value {
        Value::Object(mut obj) => {
            let delay = if let Some(delay_value) = obj.remove("delay") {
                Some(parse_delay_value(&delay_value)?)
            } else {
                None
            };

            if obj.contains_key("value")
                && !obj.contains_key("unit")
                && !obj.contains_key("units")
                && obj.len() == 1
            {
                let inner = obj
                    .remove("value")
                    .ok_or_else(|| "Expected 'value' field.".to_string())?;
                return Ok(AnswerEntry {
                    value: inner,
                    delay,
                });
            }

            Ok(AnswerEntry {
                value: Value::Object(obj),
                delay,
            })
        }
        other => Ok(AnswerEntry {
            value: other,
            delay: None,
        }),
    }
}

fn parse_delay_value(value: &Value) -> Result<chrono::Duration, String> {
    match value {
        Value::String(s) => parse_duration_arg(s),
        Value::Number(n) => {
            let secs = n
                .as_f64()
                .ok_or_else(|| "Delay must be a valid number.".to_string())?;
            if secs <= 0.0 {
                return Err("Delay must be greater than zero.".to_string());
            }
            Ok(chrono::Duration::milliseconds((secs * 1000.0) as i64))
        }
        Value::Object(obj) => {
            let amount = obj
                .get("value")
                .ok_or_else(|| "Delay object must include 'value'.".to_string())?;
            let unit = obj
                .get("unit")
                .or_else(|| obj.get("units"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Delay object must include a string 'unit'.".to_string())?;

            let numeric = match amount {
                Value::Number(n) => n
                    .as_f64()
                    .ok_or_else(|| "Delay value must be a valid number.".to_string())?,
                Value::String(s) => s
                    .parse::<f64>()
                    .map_err(|_| "Delay value must be a valid number.".to_string())?,
                _ => return Err("Delay value must be a number or string.".to_string()),
            };

            if numeric <= 0.0 {
                return Err("Delay must be greater than zero.".to_string());
            }

            let empty_units = HashMap::new();
            let unit = parse_unit_string(unit, &empty_units);
            let secs = match unit {
                Unit::Second => numeric,
                Unit::Minute => numeric * 60.0,
                Unit::Hour => numeric * 3600.0,
                Unit::Day => numeric * 86400.0,
                Unit::Week => numeric * 604800.0,
                Unit::Month => numeric * 2592000.0,
                Unit::Year => numeric * 31536000.0,
                _ => {
                    return Err(format!(
                        "Unsupported delay unit '{:?}'. Use seconds, minutes, hours, days, weeks, months, or years.",
                        unit
                    ));
                }
            };

            Ok(chrono::Duration::milliseconds((secs * 1000.0) as i64))
        }
        _ => Err("Delay must be a string or object.".to_string()),
    }
}

fn parse_string_value(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        _ => Err("Expected a string value.".to_string()),
    }
}

fn parse_quantity_value(value: &Value, unit_map: &HashMap<String, Unit>) -> Result<RuntimeValue, String> {
    match value {
        Value::Object(obj) => {
            let amount = obj
                .get("value")
                .ok_or_else(|| "Expected object with 'value' and 'unit' fields.".to_string())?;
            let unit = obj
                .get("unit")
                .or_else(|| obj.get("units"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Expected 'unit' field as a string.".to_string())?;

            let numeric = match amount {
                Value::Number(n) => n
                    .as_f64()
                    .ok_or_else(|| "Numeric value must be a valid number.".to_string())?,
                Value::String(s) => s
                    .parse::<f64>()
                    .map_err(|_| "Numeric value must be a valid number.".to_string())?,
                _ => return Err("Numeric value must be a number or string.".to_string()),
            };

            let unit = parse_unit_string(unit, unit_map);
            Ok(RuntimeValue::Quantity(numeric, unit))
        }
        Value::String(s) => {
            let (numeric, unit_str) = parse_quantity_string(s)
                .ok_or_else(|| "Numeric answers must include a unit (e.g. '10 kg').".to_string())?;
            let unit = parse_unit_string(&unit_str, unit_map);
            Ok(RuntimeValue::Quantity(numeric, unit))
        }
        Value::Number(_) => Err("Numeric answers must include a unit (e.g. '10 kg').".to_string()),
        _ => Err("Numeric answers must include a unit (e.g. '10 kg').".to_string()),
    }
}

fn parse_quantity_string(input: &str) -> Option<(f64, String)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    if let Some(first) = parts.next() {
        let rest: Vec<&str> = parts.collect();
        if !rest.is_empty() {
            if let Ok(value) = first.parse::<f64>() {
                return Some((value, rest.join(" ")));
            }
        }
    }

    let mut end_idx = 0;
    let mut saw_digit = false;
    for (idx, ch) in trimmed.char_indices() {
        if idx == 0 && (ch == '+' || ch == '-') {
            end_idx = idx + ch.len_utf8();
            continue;
        }
        if ch.is_ascii_digit() || ch == '.' {
            end_idx = idx + ch.len_utf8();
            if ch.is_ascii_digit() {
                saw_digit = true;
            }
            continue;
        }
        break;
    }

    if !saw_digit {
        return None;
    }

    let (num_str, unit_str) = trimmed.split_at(end_idx);
    let value = num_str.parse::<f64>().ok()?;
    let unit = unit_str.trim();
    if unit.is_empty() {
        return None;
    }

    Some((value, unit.to_string()))
}

fn parse_duration_arg(input: &str) -> Result<chrono::Duration, String> {
    let (value, unit_str) = parse_quantity_string(input)
        .ok_or_else(|| "Expected a duration like '30 days' or '12 hours'.".to_string())?;

    if value <= 0.0 {
        return Err("Duration must be greater than zero.".to_string());
    }

    let empty_units = HashMap::new();
    let unit = parse_unit_string(&unit_str, &empty_units);

    let secs = match unit {
        Unit::Second => value,
        Unit::Minute => value * 60.0,
        Unit::Hour => value * 3600.0,
        Unit::Day => value * 86400.0,
        Unit::Week => value * 604800.0,
        Unit::Month => value * 2592000.0,
        Unit::Year => value * 31536000.0,
        _ => {
            return Err(format!(
                "Unsupported duration unit '{}'. Use seconds, minutes, hours, days, weeks, months, or years.",
                unit_str
            ));
        }
    };

    Ok(chrono::Duration::milliseconds((secs * 1000.0) as i64))
}

fn parse_unit_string(unit_str: &str, unit_map: &HashMap<String, Unit>) -> Unit {
    let normalized = normalize_unit_name(unit_str);
    let lower = normalized.to_lowercase();

    match normalized.as_str() {
        "°C" => return Unit::Celsius,
        "°F" => return Unit::Fahrenheit,
        "mmHg" => return Unit::MillimeterOfMercury,
        "mg/dL" => return Unit::MgPerDl,
        "mmol/L" => return Unit::MmolPerL,
        "%" => return Unit::Percent,
        _ => {}
    }

    match lower.as_str() {
        "mg" | "milligram" | "milligrams" => Unit::Milligram,
        "kg" | "kilogram" | "kilograms" => Unit::Kilogram,
        "g" | "gram" | "grams" => Unit::Gram,
        "lb" | "pound" | "pounds" => Unit::Pound,
        "oz" | "ounce" | "ounces" => Unit::Ounce,
        "m" | "meter" | "meters" => Unit::Meter,
        "cm" | "centimeter" | "centimeters" => Unit::Centimeter,
        "mm" | "millimeter" | "millimeters" => Unit::Millimeter,
        "km" | "kilometer" | "kilometers" => Unit::Kilometer,
        "inch" | "inches" => Unit::Inch,
        "foot" | "feet" => Unit::Foot,
        "mile" | "miles" => Unit::Mile,
        "l" | "liter" | "liters" => Unit::Liter,
        "ml" | "milliliter" | "milliliters" => Unit::Milliliter,
        "fl oz" | "fluid ounce" | "fluid ounces" => Unit::FluidOunce,
        "gal" | "gallon" | "gallons" => Unit::Gallon,
        "celsius" => Unit::Celsius,
        "fahrenheit" => Unit::Fahrenheit,
        "mmhg" | "millimeter of mercury" => Unit::MillimeterOfMercury,
        "bpm" => Unit::Bpm,
        "mg/dl" => Unit::MgPerDl,
        "mmol/l" => Unit::MmolPerL,
        "percent" => Unit::Percent,
        "year" | "years" => Unit::Year,
        "month" | "months" => Unit::Month,
        "week" | "weeks" => Unit::Week,
        "day" | "days" => Unit::Day,
        "hour" | "hours" => Unit::Hour,
        "minute" | "minutes" => Unit::Minute,
        "second" | "seconds" => Unit::Second,
        _ => {
            if let Some(unit) = unit_map.get(&normalized) {
                return unit.clone();
            }
            if let Some(unit) = unit_map.get(&lower) {
                return unit.clone();
            }
            Unit::Custom(normalized)
        }
    }
}

fn normalize_unit_name(unit_str: &str) -> String {
    let trimmed = unit_str.trim();
    let trimmed = if trimmed.starts_with('<') && trimmed.ends_with('>') && trimmed.len() > 2 {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    trimmed.split_whitespace().collect::<Vec<&str>>().join(" ")
}
