//! FFI round-trip unit tests.
//!
//! These tests call the C FFI functions directly from Rust using `unsafe`
//! to verify correct behavior of the public FFI surface.
//!
//! NOTE: Tests that use `hippocrates_validate_file` share global state
//! (`LAST_ERRORS` mutex). Run with `--test-threads=1` to avoid interference,
//! or rely on the single grouped test below.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// A minimal valid plan used across multiple tests.
const VALID_PLAN: &str = r#"
<unit> is a unit:
    plural is <units>.

<value> is a number:
    valid values:
        0 <units> ... 10 <units>.

<test plan> is a plan:
    during plan:
        information "Hello".
"#;

/// A plan with a period definition for period-related tests.
const PLAN_WITH_PERIOD: &str = r#"
<morning> is a period:
    timeframe:
        between Monday ... Friday; 07:00 ... 09:00.
"#;

/// Invalid plan input (not valid Hippocrates syntax).
const INVALID_PLAN: &str = "this is not a valid plan at all !!!";

/// Plan with a unitless number (triggers a validation error).
const UNITLESS_NUMBER_PLAN: &str = r#"
<unit> is a unit:
    plural is <units>.

<value> is a number:
    valid values:
        0 ... 10.

<plan> is a plan:
    during plan:
        information "Hello".
"#;

// ---------------------------------------------------------------------------
// Helper: convert a raw *mut c_char to an owned String, then free via FFI.
// ---------------------------------------------------------------------------
fn read_and_free(ptr: *mut c_char) -> String {
    assert!(!ptr.is_null(), "FFI returned null pointer");
    let s = unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .expect("FFI returned invalid UTF-8")
        .to_owned();
    hippocrates_engine::ffi::hippocrates_free_string(ptr);
    s
}

// ---------------------------------------------------------------------------
// 1. hippocrates_parse_json — valid input
// ---------------------------------------------------------------------------
#[test]
fn ffi_parse_json_valid() {
    let input = CString::new(VALID_PLAN).unwrap();
    let result = hippocrates_engine::ffi::hippocrates_parse_json(input.as_ptr());
    let result_str = read_and_free(result);

    assert!(
        result_str.contains("\"Ok\""),
        "Expected Ok in parse result, got: {}",
        result_str
    );
}

// ---------------------------------------------------------------------------
// 2. hippocrates_parse_json — invalid input
// ---------------------------------------------------------------------------
#[test]
fn ffi_parse_json_invalid() {
    let input = CString::new(INVALID_PLAN).unwrap();
    let result = hippocrates_engine::ffi::hippocrates_parse_json(input.as_ptr());
    let result_str = read_and_free(result);

    assert!(
        result_str.contains("\"Err\""),
        "Expected Err in parse result, got: {}",
        result_str
    );
}

// ---------------------------------------------------------------------------
// 3 & 4. hippocrates_validate_file — valid and invalid
//
// Grouped into one test because these functions use a global LAST_ERRORS mutex.
// ---------------------------------------------------------------------------
#[test]
fn ffi_validate_file_valid_and_invalid() {
    // --- valid plan ---
    {
        let input = CString::new(VALID_PLAN).unwrap();
        let error_count = hippocrates_engine::ffi::hippocrates_validate_file(input.as_ptr());

        assert_eq!(error_count, 0, "Valid plan should have 0 errors");
    }

    // --- invalid plan (unitless number triggers validation error) ---
    {
        let input = CString::new(UNITLESS_NUMBER_PLAN).unwrap();
        let error_count = hippocrates_engine::ffi::hippocrates_validate_file(input.as_ptr());

        assert!(
            error_count > 0,
            "Unitless number plan should produce validation errors, got 0"
        );

        let stored_count = hippocrates_engine::ffi::hippocrates_get_error_count();
        assert_eq!(
            error_count, stored_count,
            "get_error_count should match validate_file return value"
        );

        // Retrieve the first error and check it is valid JSON with line/column.
        let err_ptr = hippocrates_engine::ffi::hippocrates_get_error(0);
        let err_json = read_and_free(err_ptr);

        assert!(
            err_json.contains("line") && err_json.contains("column"),
            "Error JSON should contain line and column, got: {}",
            err_json
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Engine lifecycle — new + free without crash
// ---------------------------------------------------------------------------
#[test]
fn ffi_engine_lifecycle() {
    let ctx = unsafe { hippocrates_engine::ffi::hippocrates_engine_new(std::ptr::null_mut()) };
    assert!(!ctx.is_null(), "hippocrates_engine_new returned null");
    unsafe { hippocrates_engine::ffi::hippocrates_engine_free(ctx) };
}

// ---------------------------------------------------------------------------
// 6. Engine load — valid plan
// ---------------------------------------------------------------------------
#[test]
fn ffi_engine_load_valid() {
    let ctx = unsafe { hippocrates_engine::ffi::hippocrates_engine_new(std::ptr::null_mut()) };
    let source = CString::new(VALID_PLAN).unwrap();

    let result = unsafe { hippocrates_engine::ffi::hippocrates_engine_load(ctx, source.as_ptr()) };
    let result_str = read_and_free(result);

    assert!(
        result_str.contains("\"Ok\""),
        "Expected Ok after loading valid plan, got: {}",
        result_str
    );

    unsafe { hippocrates_engine::ffi::hippocrates_engine_free(ctx) };
}

// ---------------------------------------------------------------------------
// 7. Engine load — invalid plan
// ---------------------------------------------------------------------------
#[test]
fn ffi_engine_load_invalid() {
    let ctx = unsafe { hippocrates_engine::ffi::hippocrates_engine_new(std::ptr::null_mut()) };
    let source = CString::new(INVALID_PLAN).unwrap();

    let result = unsafe { hippocrates_engine::ffi::hippocrates_engine_load(ctx, source.as_ptr()) };
    let result_str = read_and_free(result);

    assert!(
        result_str.contains("\"Err\""),
        "Expected Err after loading invalid plan, got: {}",
        result_str
    );

    unsafe { hippocrates_engine::ffi::hippocrates_engine_free(ctx) };
}

// ---------------------------------------------------------------------------
// 8. hippocrates_get_periods — load plan with period, verify JSON output
// ---------------------------------------------------------------------------
#[test]
fn ffi_get_periods() {
    let ctx = unsafe { hippocrates_engine::ffi::hippocrates_engine_new(std::ptr::null_mut()) };
    let source = CString::new(PLAN_WITH_PERIOD).unwrap();

    let load_result =
        unsafe { hippocrates_engine::ffi::hippocrates_engine_load(ctx, source.as_ptr()) };
    let load_str = read_and_free(load_result);
    assert!(
        load_str.contains("\"Ok\""),
        "Period plan should load successfully, got: {}",
        load_str
    );

    let periods_ptr = unsafe { hippocrates_engine::ffi::hippocrates_get_periods(ctx) };
    let periods_json = read_and_free(periods_ptr);

    assert!(
        periods_json.contains("morning"),
        "Periods JSON should contain the period name 'morning', got: {}",
        periods_json
    );

    unsafe { hippocrates_engine::ffi::hippocrates_engine_free(ctx) };
}

// ---------------------------------------------------------------------------
// 9. hippocrates_engine_set_time — verify no crash
// ---------------------------------------------------------------------------
#[test]
fn ffi_set_time() {
    let ctx = unsafe { hippocrates_engine::ffi::hippocrates_engine_new(std::ptr::null_mut()) };
    let source = CString::new(VALID_PLAN).unwrap();

    let result = unsafe { hippocrates_engine::ffi::hippocrates_engine_load(ctx, source.as_ptr()) };
    hippocrates_engine::ffi::hippocrates_free_string(result);

    // Set time to 2025-01-01 00:00:00 UTC (in milliseconds).
    let timestamp_ms: i64 = 1_735_689_600_000;
    unsafe { hippocrates_engine::ffi::hippocrates_engine_set_time(ctx, timestamp_ms) };

    unsafe { hippocrates_engine::ffi::hippocrates_engine_free(ctx) };
}

// ---------------------------------------------------------------------------
// 10. hippocrates_engine_enable_simulation — verify no crash
// ---------------------------------------------------------------------------
#[test]
fn ffi_enable_simulation() {
    let ctx = unsafe { hippocrates_engine::ffi::hippocrates_engine_new(std::ptr::null_mut()) };

    // Enable simulation with a 60-minute duration limit.
    unsafe { hippocrates_engine::ffi::hippocrates_engine_enable_simulation(ctx, 60) };

    unsafe { hippocrates_engine::ffi::hippocrates_engine_free(ctx) };
}

// ---------------------------------------------------------------------------
// 11. hippocrates_engine_stop — verify no crash
// ---------------------------------------------------------------------------
#[test]
fn ffi_stop() {
    let ctx = unsafe { hippocrates_engine::ffi::hippocrates_engine_new(std::ptr::null_mut()) };

    unsafe { hippocrates_engine::ffi::hippocrates_engine_stop(ctx) };

    unsafe { hippocrates_engine::ffi::hippocrates_engine_free(ctx) };
}
