#ifndef HIPPOCRATES_ENGINE_H
#define HIPPOCRATES_ENGINE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/// Parses a Hippocrates plan string and returns the AST as a JSON string.
/// The returned string must be freed using `hippocrates_free_string`.
/// Input: null-terminated UTF-8 string.
/// Output: null-terminated UTF-8 string (JSON).
char* hippocrates_parse_json(const char* input);

/// Frees a string allocated by `hippocrates_parse_json`.
void hippocrates_free_string(char* s);

// Engine Context forward declaration
typedef struct EngineContext EngineContext;

/// Creates a new EngineContext.
EngineContext* hippocrates_engine_new(void* user_data);

/// Frees the EngineContext.
void hippocrates_engine_free(EngineContext* ctx);

/// Loads a plan into the engine.
/// Returns a JSON string {"Ok": "Loaded"} on success, or {"Err": {...}} on failure.
/// The returned string must be freed using `hippocrates_free_string`.
char* hippocrates_engine_load(EngineContext* ctx, const char* source);

// Callbacks
typedef void (*LineCallback)(int, void*);
typedef void (*LogCallback)(const char*, uint8_t, int64_t, void*);
typedef void (*AskCallback)(const char*, void*);

/// Sets callbacks for the engine.
void hippocrates_engine_set_callbacks(
    EngineContext* ctx,
    LineCallback line_cb,
    LogCallback log_cb,
    AskCallback ask_cb
);

/// Executes a plan by name.
void hippocrates_engine_execute(EngineContext* ctx, const char* plan_name);

/// Sets a variable value in the engine.
/// Returns 0 on success, non-zero on failure.
int hippocrates_engine_set_value(EngineContext* ctx, const char* var_name, const char* json_val);

/// Sets the current abstract time of the engine environment.
/// Input: timestamp in milliseconds (treated as abstract local time).
void hippocrates_engine_set_time(EngineContext* ctx, int64_t timestamp_ms);

/// Enables simulation mode.
void hippocrates_engine_enable_simulation(EngineContext* ctx, int duration_mins);

/// Validates a file and returns the error count.
/// Returns 0 if valid.
int hippocrates_validate_file(const char* input);

/// Returns the number of errors from the last validation.
int hippocrates_get_error_count();

/// Returns a JSON string representation of the error at the given index.
/// Returns NULL if index is out of bounds. The string must be freed.
char* hippocrates_get_error(int index);

/// Returns a JSON string array of period definitions.
char* hippocrates_get_periods(EngineContext* ctx);

/// Returns a JSON string array of occurrence timestamps (ISO 8601).
char* hippocrates_simulate_occurrences(EngineContext* ctx, const char* period_name, int64_t start_ts, int duration_days);

/// Stops the execution of the engine.
void hippocrates_engine_stop(EngineContext* ctx);

#ifdef __cplusplus
}
#endif

#endif // HIPPOCRATES_ENGINE_H
