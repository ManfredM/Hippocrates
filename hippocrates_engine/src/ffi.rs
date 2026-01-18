use crate::parser::parse_plan;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

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
        Err(_) => return CString::new(r#"{"Err": "Invalid UTF-8 input"}"#).unwrap().into_raw(),
    };

    // Parse the input
    let result = parse_plan(input_str);

    // Serialize result to JSON
    let json_str = match result {
        Ok(plan) => {
            match serde_json::to_string(&plan) {
                Ok(s) => format!("{{\"Ok\":{}}}", s),
                Err(e) => format!("{{\"Err\":\"Serialization Error: {}\"}}", e),
            }
        },
        Err(e) => {
            let err_val = serde_json::Value::String(e.to_string());
            format!("{{\"Err\":{}}}", err_val)
        }
    };

    match CString::new(json_str) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => CString::new(r#"{"Err": "Null byte in JSON output"}"#).unwrap().into_raw(),
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
