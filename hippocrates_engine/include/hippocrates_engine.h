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

#ifdef __cplusplus
}
#endif

#endif // HIPPOCRATES_ENGINE_H
