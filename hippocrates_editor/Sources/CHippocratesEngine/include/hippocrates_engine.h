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
typedef void (*LogCallback)(const char*, uint8_t, int64_t, void*);
typedef void (*AskCallback)(const char* request_json, void* user_data);

typedef struct EngineContext EngineContext;

EngineContext* hippocrates_engine_new(void* user_data);
void hippocrates_engine_free(EngineContext* ctx);
int hippocrates_engine_load(EngineContext* ctx, const char* source);
void hippocrates_engine_set_callbacks(
    EngineContext* ctx, 
    LineCallback line_cb, 
    LogCallback log_cb,
    AskCallback ask_cb
);
void hippocrates_engine_execute(EngineContext* ctx, const char* plan_name);
int hippocrates_engine_set_value(EngineContext* ctx, const char* var_name, const char* json_val);

void hippocrates_run(
    const char* input,
    const char* plan_name,
    LineCallback callback,
    LogCallback log_callback,
    void* user_data
);

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
