use crate::parser::parse_plan;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Parses a Hippocrates plan string and returns the AST as a JSON string.
/// The returned string must be freed using `hippocrates_free_string`.
/// Returns a JSON object with either {"Ok": ... } or {"Err": ... } structure,
/// or a simple error string if serialization fails (unlikely).
#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_parse_json(input: *const c_char) -> *mut c_char {
    let c_str = unsafe {
        if input.is_null() {
            return std::ptr::null_mut();
        }
        CStr::from_ptr(input)
    };

    let input_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new(r#"{"Err": "Invalid UTF-8 input"}"#)
                .unwrap()
                .into_raw();
        }
    };

    // Parse the input
    let result = parse_plan(input_str);

    // Serialize result to JSON
    let json_str = match result {
        Ok(plan) => match serde_json::to_string(&plan) {
            Ok(s) => format!("{{\"Ok\":{}}}", s),
            Err(e) => format!("{{\"Err\":\"Serialization Error: {}\"}}", e),
        },
        Err(e) => {
            let err_val = serde_json::Value::String(e.to_string());
            format!("{{\"Err\":{}}}", err_val)
        }
    };

    match CString::new(json_str) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => CString::new(r#"{"Err": "Null byte in JSON output"}"#)
            .unwrap()
            .into_raw(),
    }
}

/// Frees a string allocated by `hippocrates_parse_json`.
#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(s));
    }
}

pub type LineCallback = extern "C" fn(c_int, *mut std::ffi::c_void);
pub type LogCallback = extern "C" fn(*const c_char, i64, *mut std::ffi::c_void);

struct SendPtr(*mut std::ffi::c_void);
unsafe impl Send for SendPtr {}
unsafe impl Sync for SendPtr {}

impl SendPtr {
    fn get(&self) -> *mut std::ffi::c_void {
        self.0
    }
}

/// Executes a plan by name from the provided source code.
/// Calls:
/// - `callback` with the line number.
/// - `log_callback` with execution logs.
#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_run(
    input: *const c_char,
    plan_name: *const c_char,
    callback: Option<LineCallback>,
    log_callback: Option<LogCallback>,
    user_data: *mut std::ffi::c_void,
) {
    hippocrates_execute_internal(input, plan_name, callback, log_callback, user_data, None);
}

#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_simulate(
    input: *const c_char,
    plan_name: *const c_char,
    callback: Option<LineCallback>,
    log_callback: Option<LogCallback>,
    user_data: *mut std::ffi::c_void,
    days: c_int,
) {
    hippocrates_execute_internal(
        input,
        plan_name,
        callback,
        log_callback,
        user_data,
        Some(days),
    );
}

fn hippocrates_execute_internal(
    input: *const c_char,
    plan_name: *const c_char,
    callback: Option<LineCallback>,
    log_callback: Option<LogCallback>,
    user_data: *mut std::ffi::c_void,
    simulation_days: Option<c_int>,
) {
    let input_str = unsafe {
        if input.is_null() {
            return;
        }
        match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return,
        }
    };

    let plan_name_str = unsafe {
        if plan_name.is_null() {
            return;
        }
        match CStr::from_ptr(plan_name).to_str() {
            Ok(s) => s,
            Err(_) => return,
        }
    };

    // Parse
    let plan = match parse_plan(input_str) {
        Ok(p) => p,
        Err(e) => {
            println!("FFI Execution Error: Parse failed: {}", e);
            return;
        }
    };

    // Setup Runtime
    let mut env = crate::runtime::Environment::new();
    env.load_plan(plan);

    let line_cb_wrapper: Box<dyn Fn(usize) + Send> = if let Some(cb) = callback {
        let ptr = SendPtr(user_data);
        Box::new(move |line: usize| {
            cb(line as c_int, ptr.get());
        })
    } else {
        Box::new(|_| {})
    };

    let log_cb_wrapper: Box<dyn Fn(String, chrono::DateTime<chrono::Utc>) + Send> =
        if let Some(cb) = log_callback {
            let ptr = SendPtr(user_data);
            Box::new(move |msg: String, ts: chrono::DateTime<chrono::Utc>| {
                if let Ok(c_msg) = CString::new(msg) {
                    cb(c_msg.as_ptr(), ts.timestamp_millis(), ptr.get());
                }
            })
        } else {
            Box::new(|_, _| {})
        };

    let mut executor = crate::runtime::Executor::with_activites(line_cb_wrapper, log_cb_wrapper);
    if let Some(days) = simulation_days {
        executor.set_mode(crate::runtime::ExecutionMode::Simulation(
            std::time::Duration::from_secs((days as u64) * 86400),
        ));
    }

    executor.execute_plan(&mut env, plan_name_str);
}
