use crate::parser::parse_plan;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Parses a Hippocrates plan string and returns the AST as a JSON string.
/// The returned string must be freed using `hippocrates_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn hippocrates_parse_json(input: *const c_char) -> *mut c_char {
    let c_str = unsafe {
        if input.is_null() { return std::ptr::null_mut(); }
        CStr::from_ptr(input)
    };
    let input_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new(r#"{"Err": "Invalid UTF-8 input"}"#).unwrap().into_raw(),
    };
    let result = parse_plan(input_str);
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
        Err(_) => CString::new(r#"{"Err": "Null byte"}"#).unwrap().into_raw(),
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
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hippocrates_engine_new(user_data: *mut std::ffi::c_void) -> *mut EngineContext {
    let env = crate::runtime::Environment::new();
    let mut executor = crate::runtime::Executor::new();
    let (tx, rx) = std::sync::mpsc::channel();
    executor.set_input_receiver(rx);
    
    let ctx = std::boxed::Box::new(EngineContext {
        env,
        executor,
        user_data,
        input_sender: tx,
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
pub unsafe extern "C" fn hippocrates_engine_load(ctx: *mut EngineContext, source: *const c_char) -> c_int { unsafe {
    let context = &mut *ctx;
    let c_str = CStr::from_ptr(source);
    let s = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match parse_plan(s) {
        Ok(plan) => {
            context.env.load_plan(plan);
            0
        }
        Err(_) => 1,
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

    if let Some(cb) = line_cb {
        context.executor.on_step = Some(Box::new(move |line| {
            cb(line as c_int, ptr.get());
        }));
    }
    if let Some(cb) = log_cb {
        context.executor.on_log = Some(Box::new(move |msg, event_type, ts| {
            if let Ok(c_msg) = CString::new(msg) {
                cb(c_msg.as_ptr(), event_type as u8, ts.timestamp_millis(), ptr2.get());
            }
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
    var_name: *const c_char, 
    json_val: *const c_char
) -> c_int { unsafe {
    let context = &mut *ctx;
    let name = match CStr::from_ptr(var_name).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let json = match CStr::from_ptr(json_val).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    // Look up variable definition to know expected type
    let expected_type = if let Some(crate::ast::Definition::Value(def)) = context.env.definitions.get(name) {
        Some(def.value_type.clone())
    } else {
        None
    };

    let runtime_val = match expected_type {
        Some(crate::domain::ValueType::Number) => {
             // Try to parse as f64
             if let Ok(n) = serde_json::from_str::<f64>(json) {
                 Some(crate::domain::RuntimeValue::Number(n))
             } else if let Ok(s) = serde_json::from_str::<String>(json) {
                 // Maybe it came as string "10"?
                 if let Ok(n) = s.parse::<f64>() {
                     Some(crate::domain::RuntimeValue::Number(n))
                 } else { None }
             } else { None }
        },
        Some(crate::domain::ValueType::Enumeration) | Some(crate::domain::ValueType::Addressee) => {
            // Expect string
             if let Ok(s) = serde_json::from_str::<String>(json) {
                 Some(crate::domain::RuntimeValue::Enumeration(s))
             } else { None }
        },
        // TODO: Handle other types
        _ => {
            // Fallback to trying to guess or naive deserialization?
            // For now, let's try direct deserialize if we don't know type, 
            // but usually we should know.
            serde_json::from_str::<crate::domain::RuntimeValue>(json).ok()
        }
    };

    if let Some(val) = runtime_val {
        // Use channel to handle both pre-exec and mid-exec updates safely
        let msg = crate::domain::InputMessage {
            variable: name.to_string(),
            value: val,
        };
        match context.input_sender.send(msg) {
            Ok(_) => 0,
            Err(_) => 1 // Sender disconnected
        }
    } else {
        1 // Failed to parse or unknown variable
    }
}}

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
        if hippocrates_engine_load(ctx, input) == 0 {
            hippocrates_engine_execute(ctx, plan_name);
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
        if hippocrates_engine_load(ctx, input) == 0 {
            hippocrates_engine_execute(ctx, plan_name);
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
