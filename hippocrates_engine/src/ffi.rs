use crate::parser::parse_plan;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use serde::Serialize;
use chrono::DateTime;
use crate::runtime::scheduler::Scheduler;

// Parses a Hippocrates plan string and returns the AST as a JSON string.
/// The returned string must be freed using `hippocrates_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_parse_json(input: *const c_char) -> *mut c_char {
    let c_str = unsafe {
        if input.is_null() { return std::ptr::null_mut(); }
        CStr::from_ptr(input)
    };
    let input_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new(r#"{"Err": {"message": "Invalid UTF-8 input", "line": 0, "column": 0}}"#).unwrap().into_raw(),
    };
    let result = parse_plan(input_str);
    let json_str = match result {
        Ok(plan) => match serde_json::to_string(&plan) {
            Ok(s) => format!("{{\"Ok\":{}}}", s),
            Err(e) => format!("{{\"Err\":{{\"message\": \"Serialization Error: {}\", \"line\": 0, \"column\": 0}}}}", e),
        },
        Err(e) => {
            // EngineError serializes to {message, line, column}
            match serde_json::to_string(&e) {
                Ok(s) => format!("{{\"Err\":{}}}", s),
                Err(_) => r#"{"Err": {"message": "Error serialization failed", "line": 0, "column": 0}}"#.to_string(),
            }
        }
    };
    match CString::new(json_str) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => CString::new(r#"{"Err": {"message": "Null byte", "line": 0, "column": 0}}"#).unwrap().into_raw(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_free_string(s: *mut c_char) {
    if !s.is_null() { unsafe { drop(CString::from_raw(s)); } }
}

// Callbacks
pub type LineCallback = extern "C" fn(c_int, *mut std::ffi::c_void);
pub type LogCallback = extern "C" fn(*const c_char, u8, i64, *mut std::ffi::c_void);
pub type AskCallback = extern "C" fn(*const c_char, *mut std::ffi::c_void);

struct SendPtr(*mut std::ffi::c_void);
unsafe impl Send for SendPtr {}
unsafe impl Sync for SendPtr {}
impl SendPtr { fn get(&self) -> *mut std::ffi::c_void { self.0 } }

pub struct EngineContext {
    pub env: crate::runtime::Environment,
    pub executor: crate::runtime::Executor,
    pub user_data: *mut std::ffi::c_void,
    pub input_sender: std::sync::mpsc::Sender<crate::domain::InputMessage>,
    pub stop_signal: Arc<AtomicBool>,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_engine_new(user_data: *mut std::ffi::c_void) -> *mut EngineContext {
    let env = crate::runtime::Environment::new();
    let stop_signal = Arc::new(AtomicBool::new(false));
    let mut executor = crate::runtime::Executor::new(stop_signal.clone());
    let (tx, rx) = std::sync::mpsc::channel();
    executor.set_input_receiver(rx);
    
    let ctx = std::boxed::Box::new(EngineContext {
        env,
        executor,
        user_data,
        input_sender: tx,
        stop_signal,
    });
    Box::into_raw(ctx)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_engine_free(ctx: *mut EngineContext) { unsafe {
    if !ctx.is_null() {
        drop(Box::from_raw(ctx));
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_engine_load(ctx: *mut EngineContext, source: *const c_char) -> *mut c_char { unsafe {
    let context = &mut *ctx;
    let c_str = CStr::from_ptr(source);
    let s = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new(r#"{"Err": {"message": "Invalid UTF-8 input", "line": 0, "column": 0}}"#).unwrap().into_raw(),
    };
    match parse_plan(s) {
        Ok(plan) => {
            // Validate before loading
            if let Err(errors) = crate::runtime::validator::validate_file(&plan) {
                 // For legacy load, return the first error
                 if let Some(first) = errors.first() {
                     match serde_json::to_string(first) {
                        Ok(s) => return CString::new(format!("{{\"Err\":{}}}", s)).unwrap().into_raw(),
                        Err(_) => return CString::new(r#"{"Err": {"message": "Error serialization failed", "line": 0, "column": 0}}"#).unwrap().into_raw(),
                    }
                 }
            }
            context.env.load_plan(plan);
            CString::new(r#"{"Ok": "Loaded"}"#).unwrap().into_raw()
        }
        Err(e) => {
            match serde_json::to_string(&e) {
                Ok(s) => CString::new(format!("{{\"Err\":{}}}", s)).unwrap().into_raw(),
                Err(_) => CString::new(r#"{"Err": {"message": "Error serialization failed", "line": 0, "column": 0}}"#).unwrap().into_raw(),
            }
        }
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_get_periods(ctx: *mut EngineContext) -> *mut c_char { unsafe {
    if ctx.is_null() { return std::ptr::null_mut(); }
    let context = &*ctx;
    let mut periods = Vec::new();
    for def in context.env.definitions.values() {
        if let crate::ast::Definition::Period(period) = def {
            periods.push(period);
        }
    }
    
    match serde_json::to_string(&periods) {
        Ok(s) => match CString::new(s) {
            Ok(c) => c.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}}

#[derive(Serialize)]
struct Occur {
    start: String,
    end: String,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_simulate_occurrences(
    ctx: *mut EngineContext,
    period_name: *const c_char,
    start_ts: i64,
    duration_days: c_int,
) -> *mut c_char { unsafe {
    if ctx.is_null() || period_name.is_null() {
        return std::ptr::null_mut();
    }

    let context = &mut *ctx;
    let name_c = CStr::from_ptr(period_name);
    let name = match name_c.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let def = match context.env.definitions.get(name) {
        Some(crate::ast::Definition::Period(p)) => crate::ast::Definition::Period(p.clone()),
        _ => return std::ptr::null_mut(),
    };

    let mut occurrences = Vec::new();
    let start_dt = DateTime::from_timestamp_millis(start_ts).unwrap().naive_utc();
    
    let end_limit = start_dt + chrono::Duration::days(duration_days as i64);
    let mut current_time = start_dt;
    
    // Loop to find occurrences
    // Limit to reasonable count to prevent infinite loop
    for _ in 0..100 {
        if current_time >= end_limit { break; }
        
        if let Some((start, end)) = Scheduler::next_occurrence(&def, current_time) {
            // Ensure we advance time
            if start <= current_time {
                 current_time = current_time + chrono::Duration::minutes(1);
                 continue;
            }
            
            occurrences.push(Occur {
                start: start.and_utc().to_rfc3339(),
                end: end.and_utc().to_rfc3339()
            });
            current_time = start; // Advance to start of this occurrence
        } else {
            break;
        }
    }
    
    match serde_json::to_string(&occurrences) {
        Ok(s) => match CString::new(s) {
            Ok(c) => c.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}}



#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_engine_set_callbacks(
    ctx: *mut EngineContext,
    line_cb: Option<LineCallback>,
    log_cb: Option<LogCallback>,
    ask_cb: Option<AskCallback>,
) { unsafe {
    let context = &mut *ctx;
    let ptr = SendPtr(context.user_data);
    let ptr2 = SendPtr(context.user_data);
    let ptr3 = SendPtr(context.user_data);
    // let ptr4 = SendPtr(context.user_data);

    if let Some(cb) = line_cb {
        context.executor.on_step = Some(Box::new(move |line| {
            cb(line as c_int, ptr.get());
        }));
    }
    if let Some(cb) = log_cb {
        // 1. Set Executor log handler (for typed events)
        context.executor.on_log = Some(Box::new(move |msg, event_type, ts| {
            if let Ok(c_msg) = CString::new(msg) {
                cb(c_msg.as_ptr(), event_type as u8, ts.and_utc().timestamp_millis(), ptr2.get());
            }
        }));

        // 2. Set Environment log handler (for debug/plain logs)
        // We use EventType::Log (4 or 0? 0 is Log in Swift default, 1=Message, 2=Question, 3=Answer)
        // Swift switch default is Log.
        context.env.set_output_handler(std::sync::Arc::new(move |msg: String| {
                // cb(c_msg.as_ptr(), 0, chrono::Utc::now().timestamp_millis(), ptr4.get());
            // }
            println!("ENGINE LOG: {}", msg);
        }));
    }
    if let Some(cb) = ask_cb {
        context.executor.set_ask_callback(Box::new(move |req| {
            if let Ok(json) = serde_json::to_string(&req) {
                 if let Ok(c_json) = CString::new(json) {
                     cb(c_json.as_ptr(), ptr3.get());
                 }
            }
        }));
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_engine_execute(ctx: *mut EngineContext, plan_name: *const c_char) { unsafe {
    let context = &mut *ctx;
    let c_str = CStr::from_ptr(plan_name);
    if let Ok(name) = c_str.to_str() {
        context.executor.execute_plan(&mut context.env, name);
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_engine_set_value(
    ctx: *mut EngineContext,
    name_ptr: *const c_char,
    json_ptr: *const c_char
) -> c_int {
    if ctx.is_null() || name_ptr.is_null() || json_ptr.is_null() {
        return 1;
    }

    let context = unsafe { &mut *ctx };
    let name = unsafe {
        match CStr::from_ptr(name_ptr).to_str() {
            Ok(s) => s,
            Err(_) => return 1,
        }
    };
    let json = unsafe {
        match CStr::from_ptr(json_ptr).to_str() {
            Ok(s) => s,
            Err(_) => return 1,
        }
    };

    // Determine type from definition
    let expected_type = context.env.definitions.get(name).and_then(|def| {
        if let crate::ast::Definition::Value(v) = def {
            Some(v.value_type.clone())
        } else {
            None
        }
    });

    let runtime_val = match expected_type {
        Some(crate::domain::ValueType::Number) => {
             if let Ok(n) = serde_json::from_str::<f64>(json) {
                 Some(crate::domain::RuntimeValue::Number(n))
             } else if let Ok(s) = serde_json::from_str::<String>(json) {
                  let parts: Vec<&str> = s.split_whitespace().collect();
                  if parts.len() >= 2 {
                       if let Ok(n) = parts[0].parse::<f64>() {
                           let unit_str = parts[1].trim_matches(|c| c == ':' || c == ',' || c == ';');
                           let unit = crate::domain::Unit::Custom(unit_str.to_string());
                           Some(crate::domain::RuntimeValue::Quantity(n, unit))
                       } else {
                           Some(crate::domain::RuntimeValue::String(s))
                       }
                  } else {
                       if let Ok(n) = s.parse::<f64>() {
                           Some(crate::domain::RuntimeValue::Number(n))
                       } else {
                           Some(crate::domain::RuntimeValue::String(s))
                       }
                  }
             } else { None }
        },
        _ => match serde_json::from_str::<crate::domain::RuntimeValue>(json) {
            Ok(v) => Some(v),
            Err(_) => {
                 if let Ok(s) = serde_json::from_str::<String>(json) {
                     Some(crate::domain::RuntimeValue::String(s))
                 } else {
                     None
                 }
            }
        }
    };

    if let Some(val) = runtime_val {
        let msg = crate::domain::InputMessage {
            variable: name.to_string(),
            value: val,
            timestamp: chrono::Utc::now().naive_utc(),
        };
        match context.input_sender.send(msg) {
            Ok(_) => 0,
            Err(_) => 1 
        }
    } else {
        1 
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_engine_set_time(
    ctx: *mut EngineContext,
    timestamp_ms: i64,
) {
    let ctx = unsafe { &mut *ctx };
    if let Some(dt) = DateTime::from_timestamp(timestamp_ms / 1000, 0) {
        ctx.env.set_time(dt.naive_utc());
    }
}



// Legacy helpers (rewritten to use EngineContext)
#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_run(
    input: *const c_char,
    plan_name: *const c_char,
    callback: Option<LineCallback>,
    log_callback: Option<LogCallback>,
    user_data: *mut std::ffi::c_void,
) {
    unsafe {
        let ctx = hippocrates_engine_new(user_data);
        hippocrates_engine_set_callbacks(ctx, callback, log_callback, None);
        
        // Handle new return type of load
        let result = hippocrates_engine_load(ctx, input);
        if !result.is_null() {
            let s = CStr::from_ptr(result).to_string_lossy();
            if s.contains("\"Ok\"") {
                hippocrates_engine_execute(ctx, plan_name);
            } else {
                // If log callback exists, maybe log the error?
                // For now just ignore as this is legacy.
                if let Some(cb) = log_callback {
                    // Try to extract message
                     if let Ok(c_msg) = CString::new(format!("Load Error: {}", s)) {
                        cb(c_msg.as_ptr(), 0, 0, user_data);
                    }
                }
            }
            hippocrates_free_string(result);
        }
        
        hippocrates_engine_free(ctx);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_simulate(
    input: *const c_char,
    plan_name: *const c_char,
    callback: Option<LineCallback>,
    log_callback: Option<LogCallback>,
    user_data: *mut std::ffi::c_void,
    _days: c_int,
) {
    unsafe {
        let ctx = hippocrates_engine_new(user_data);
        hippocrates_engine_set_callbacks(ctx, callback, log_callback, None);
        (*ctx).executor.set_mode(crate::runtime::ExecutionMode::Simulation {
            speed_factor: None, // Instant execution / max speed
            duration: None,
        });
        
        // Handle new return type of load
        let result = hippocrates_engine_load(ctx, input);
        if !result.is_null() {
            let s = CStr::from_ptr(result).to_string_lossy();
            if s.contains("\"Ok\"") {
                 hippocrates_engine_execute(ctx, plan_name);
            } else {
                 if let Some(cb) = log_callback {
                     if let Ok(c_msg) = CString::new(format!("Load Error: {}", s)) {
                        cb(c_msg.as_ptr(), 0, 0, user_data);
                    }
                }
            }
             hippocrates_free_string(result);
        }
        
        hippocrates_engine_free(ctx);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_engine_enable_simulation(ctx: *mut EngineContext, duration_mins: c_int) {
    unsafe {
        let context = &mut *ctx;
        let limit = if duration_mins > 0 {
            Some(chrono::Duration::minutes(duration_mins as i64))
        } else {
            None
        };
        context
            .executor
            .set_mode(crate::runtime::ExecutionMode::Simulation { speed_factor: None, duration: limit });
    }
}

// Validation FFI

use std::sync::Mutex;

static LAST_ERRORS: Mutex<Vec<crate::domain::EngineError>> = Mutex::new(Vec::new());

#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_validate_file(input: *const c_char) -> c_int {
    let input_str = unsafe {
        if input.is_null() { return 0; }
        match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };

    // lock and clear errors
    let mut errors_guard = LAST_ERRORS.lock().unwrap();
    errors_guard.clear();

    match parse_plan(input_str) {
        Ok(plan) => {
             match crate::runtime::validator::validate_file(&plan) {
                 Ok(_) => 0,
                 Err(errs) => {
                     let count = errs.len() as c_int;
                     *errors_guard = errs;
                     count
                 }
             }
        },
        Err(e) => {
            // Parser error is single, but treat as one error
            errors_guard.push(e);
            1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_get_error_count() -> c_int {
    let errors_guard = LAST_ERRORS.lock().unwrap();
    errors_guard.len() as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_get_error(index: c_int) -> *mut c_char {
    let errors_guard = LAST_ERRORS.lock().unwrap();
    if index < 0 || index as usize >= errors_guard.len() {
        return std::ptr::null_mut();
    }
    
    let err = &errors_guard[index as usize];
    match serde_json::to_string(err) {
        Ok(s) => match CString::new(s) {
            Ok(c) => c.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_engine_stop(ctx: *mut EngineContext) { unsafe {
    let context = &mut *ctx;
    context.executor.stop_signal.store(true, Ordering::SeqCst);
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_get_audit_log(ctx: *mut EngineContext) -> *mut c_char { unsafe {
    if ctx.is_null() {
        return std::ptr::null_mut();
    }
    let context = &mut *ctx;
    match serde_json::to_string(&context.env.audit_log) {
        Ok(s) => match CString::new(s) {
            Ok(c) => c.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}}
