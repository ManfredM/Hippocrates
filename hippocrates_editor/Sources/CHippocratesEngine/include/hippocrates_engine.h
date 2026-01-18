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

typedef void (*LineCallback)(int, void* user_data);
typedef void (*LogCallback)(const char*, int64_t, void*);

/// Executes a plan by name from the provided source code.
/// Calls the `callback` with the line number of each statement executed.
void hippocrates_run(
    const char* input,
    const char* plan_name,
    LineCallback callback,
    LogCallback log_callback,
    void* user_data
);

/// Simulates a plan execution over a specified number of days.
/// Fast-forwards time without sleeping.
void hippocrates_simulate(
    const char* input,
    const char* plan_name,
    LineCallback callback,
    LogCallback log_callback,
    void* user_data,
    int days
);

#ifdef __cplusplus
}
#endif

#endif // HIPPOCRATES_ENGINE_H
