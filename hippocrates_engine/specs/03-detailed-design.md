# Detailed Design Document

| Field         | Value                                      |
|---------------|--------------------------------------------|
| Document ID   | DDR                                        |
| V-Model Level | Detailed Design                            |
| Version       | 1.0                                        |
| Status        | Draft                                      |
| Date          | 2026-03-20                                 |

---

## 1. Module Inventory

The engine source tree is rooted at `hippocrates_engine/src/` and is organized as follows.

| Module Path                              | Responsibility                                  |
|------------------------------------------|--------------------------------------------------|
| `src/lib.rs`                             | Crate root; re-exports public API                |
| `src/main.rs`                            | CLI entry point (binary target)                  |
| `src/ast.rs`                             | AST node type definitions                        |
| `src/domain.rs`                          | Domain model types (RuntimeValue, Unit, etc.)    |
| `src/grammar.pest`                       | PEG grammar consumed by `pest`                   |
| `src/parser.rs`                          | Indentation preprocessor and pest-to-AST builder |
| `src/ffi.rs`                             | C-FFI layer (`extern "C"` functions)             |
| `src/formatter.rs`                       | Source code pretty-printer                        |
| `src/runtime/mod.rs`                     | Runtime module root; Engine struct, helpers       |
| `src/runtime/environment.rs`             | Value store, definitions, context stack           |
| `src/runtime/executor.rs`               | Event-loop executor with BinaryHeap scheduler     |
| `src/runtime/evaluator.rs`              | Expression evaluator                              |
| `src/runtime/scheduler.rs`              | Period occurrence calculator                       |
| `src/runtime/session.rs`                | Multi-plan session coordinator                     |
| `src/runtime/input_validation.rs`       | Runtime answer precision validation                |
| `src/runtime/validator/mod.rs`          | Validation pipeline entry point                    |
| `src/runtime/validator/semantics.rs`    | Semantic checks on definitions and statements      |
| `src/runtime/validator/intervals.rs`    | Numeric interval arithmetic                        |
| `src/runtime/validator/data_flow.rs`    | Use-before-assignment analysis                     |
| `src/runtime/validator/coverage.rs`     | Assessment exhaustiveness checking                 |

### Dependency Overview

```
ffi.rs --> parser.rs --> grammar.pest
  |            |
  v            v
runtime/       ast.rs
  mod.rs       domain.rs
  |
  +-- environment.rs
  +-- executor.rs  --> evaluator.rs
  |                --> scheduler.rs
  +-- session.rs   --> executor.rs, environment.rs
  +-- input_validation.rs
  +-- validator/
        mod.rs --> semantics.rs, intervals.rs,
                   data_flow.rs, coverage.rs
formatter.rs --> parser.rs (reuses preprocessor and pest parser)
```

---

## 2. C-FFI Interface

All public C functions are declared in `include/hippocrates_engine.h` and implemented in `src/ffi.rs`.

### Parsing and Validation

#### DDR-FFI-01: hippocrates_parse_json

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `char* hippocrates_parse_json(const char* input)` |
| **Purpose**     | Parses a Hippocrates plan source string and returns the AST serialized as JSON. |
| **Preconditions** | `input` is a non-null, null-terminated UTF-8 string containing valid or invalid plan source. |
| **Postconditions** | Returns a heap-allocated JSON string. On success: `{"Ok": <ast>}`. On error: `{"Err": {"message": "...", "line": N, "column": N}}`. |
| **Error handling** | Returns a JSON error object for invalid UTF-8, parse failures, and serialization errors. Never returns null for valid input. Returns null only if `input` is null. |
| **Memory**      | Caller must free the returned string via `hippocrates_free_string`. |

#### DDR-FFI-02: hippocrates_free_string

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `void hippocrates_free_string(char* s)` |
| **Purpose**     | Frees a string previously allocated by any `hippocrates_*` function that returns `char*`. |
| **Preconditions** | `s` was returned by a Hippocrates FFI function, or is null. |
| **Postconditions** | The memory is deallocated. Passing null is a no-op. |
| **Error handling** | None. |
| **Memory**      | After this call, `s` is invalid and must not be dereferenced. |

#### DDR-FFI-03: hippocrates_validate_file

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `int hippocrates_validate_file(const char* input)` |
| **Purpose**     | Parses and validates a plan source string. Stores errors in a global `LAST_ERRORS` vector. |
| **Preconditions** | `input` is a non-null, null-terminated UTF-8 string. |
| **Postconditions** | Returns 0 if the plan is valid. Returns the error count (>0) if validation fails. Errors are retrievable via `hippocrates_get_error_count` and `hippocrates_get_error`. |
| **Error handling** | Parse errors are treated as a single error. Validation errors are accumulated. Returns 0 for null input. |
| **Memory**      | Errors are stored in a global `Mutex<Vec<EngineError>>`. Thread-safe but shared across calls. |

#### DDR-FFI-04: hippocrates_get_error_count

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `int hippocrates_get_error_count()` |
| **Purpose**     | Returns the number of errors from the last `hippocrates_validate_file` call. |
| **Preconditions** | A prior call to `hippocrates_validate_file` has been made. |
| **Postconditions** | Returns the count of stored errors. |
| **Error handling** | None. Returns 0 if no validation has been performed. |
| **Memory**      | No allocation. |

#### DDR-FFI-05: hippocrates_get_error

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `char* hippocrates_get_error(int index)` |
| **Purpose**     | Returns a JSON representation of the error at the given index from the last validation. |
| **Preconditions** | `index` is in range `[0, hippocrates_get_error_count())`. |
| **Postconditions** | Returns a JSON string `{"message": "...", "line": N, "column": N}`. Returns null if index is out of bounds. |
| **Error handling** | Returns null for out-of-bounds index or serialization failure. |
| **Memory**      | Caller must free via `hippocrates_free_string`. |

### Engine Lifecycle

#### DDR-FFI-06: hippocrates_engine_new

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `EngineContext* hippocrates_engine_new(void* user_data)` |
| **Purpose**     | Creates a new `EngineContext` containing an `Environment`, `Executor`, input channel, and stop signal. |
| **Preconditions** | `user_data` is an opaque pointer passed through to callbacks. May be null. |
| **Postconditions** | Returns a heap-allocated `EngineContext`. The executor has a connected `mpsc` channel for input messages. |
| **Error handling** | None (infallible). |
| **Memory**      | Caller must free via `hippocrates_engine_free`. The context owns the `Environment`, `Executor`, and channel sender. |

#### DDR-FFI-07: hippocrates_engine_free

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `void hippocrates_engine_free(EngineContext* ctx)` |
| **Purpose**     | Deallocates an `EngineContext` and all owned resources. |
| **Preconditions** | `ctx` was returned by `hippocrates_engine_new`, or is null. |
| **Postconditions** | All memory is freed. Passing null is a no-op. |
| **Error handling** | None. |
| **Memory**      | Drops the `Box<EngineContext>` and all fields. |

### Loading

#### DDR-FFI-08: hippocrates_engine_load

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `char* hippocrates_engine_load(EngineContext* ctx, const char* source)` |
| **Purpose**     | Parses a plan source string, validates it, and loads definitions into the engine environment. |
| **Preconditions** | `ctx` is a valid `EngineContext`. `source` is a non-null, null-terminated UTF-8 string. |
| **Postconditions** | On success, returns `{"Ok": "Loaded"}` and the environment contains all definitions with default values initialized. On failure, returns `{"Err": {...}}` with the first error. |
| **Error handling** | Returns JSON error for parse failures, UTF-8 errors, and validation errors (first error only). |
| **Memory**      | Caller must free the returned string via `hippocrates_free_string`. |

### Callback Registration

#### DDR-FFI-09: hippocrates_engine_set_callbacks

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `void hippocrates_engine_set_callbacks(EngineContext* ctx, LineCallback line_cb, LogCallback log_cb, AskCallback ask_cb)` |
| **Purpose**     | Registers callback functions for execution step tracking, event logging, and question prompts. |
| **Preconditions** | `ctx` is a valid `EngineContext`. Callbacks may be null (Optional). |
| **Postconditions** | `line_cb` is called with `(line_number, user_data)` on each statement execution step. `log_cb` is called with `(json_message, event_type, timestamp_ms, user_data)` for audit events. `ask_cb` is called with `(json_ask_request, user_data)` when the engine needs user input. |
| **Error handling** | None. Null callbacks are silently ignored. |
| **Memory**      | Callbacks are stored as boxed closures in the executor. The `user_data` pointer is captured by value (wrapped in `SendPtr` for thread safety). |

Callback type signatures:
- `LineCallback`: `void (*)(int line, void* user_data)`
- `LogCallback`: `void (*)(const char* json, uint8_t event_type, int64_t timestamp_ms, void* user_data)`
- `AskCallback`: `void (*)(const char* json, void* user_data)`

#### DDR-FFI-10: hippocrates_engine_set_message_callback

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `void hippocrates_engine_set_message_callback(EngineContext* ctx, MessageCallback message_cb)` |
| **Purpose**     | Registers a dedicated callback for message delivery payloads (information, warning, urgent warning). |
| **Preconditions** | `ctx` is a valid, non-null `EngineContext`. |
| **Postconditions** | `message_cb` is called with `(json_payload, timestamp_ms, user_data)` when the engine produces a message action. Setting null clears the callback. |
| **Error handling** | None. Returns silently for null `ctx`. |
| **Memory**      | Callback stored as boxed closure in executor. |

Callback type signature:
- `MessageCallback`: `void (*)(const char* json, int64_t timestamp_ms, void* user_data)`

### Execution and Control

#### DDR-FFI-11: hippocrates_engine_execute

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `void hippocrates_engine_execute(EngineContext* ctx, const char* plan_name)` |
| **Purpose**     | Begins executing a named plan. This is a blocking call that runs the event loop until completion or stop signal. |
| **Preconditions** | `ctx` is a valid `EngineContext` with a loaded plan. `plan_name` matches a plan definition name. |
| **Postconditions** | The plan executes: `before plan` blocks run immediately, periodic triggers are scheduled, the event loop processes events in chronological order, and `after plan` blocks run on completion. |
| **Error handling** | Silently returns if `plan_name` is invalid UTF-8 or does not match a known plan. |
| **Memory**      | No allocation returned to caller. |

#### DDR-FFI-12: hippocrates_engine_stop

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `void hippocrates_engine_stop(EngineContext* ctx)` |
| **Purpose**     | Sets the stop signal to terminate the executor event loop. |
| **Preconditions** | `ctx` is a valid `EngineContext`. |
| **Postconditions** | The `stop_signal` `AtomicBool` is set to `true` with `SeqCst` ordering. The event loop will exit at the next check point. |
| **Error handling** | None. |
| **Memory**      | No allocation. |

#### DDR-FFI-13: hippocrates_engine_enable_simulation

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `void hippocrates_engine_enable_simulation(EngineContext* ctx, int duration_mins)` |
| **Purpose**     | Switches the executor to simulation mode with optional time-bounded duration. |
| **Preconditions** | `ctx` is a valid `EngineContext`. Must be called before `hippocrates_engine_execute`. |
| **Postconditions** | Executor mode is set to `ExecutionMode::Simulation { speed_factor: None, duration }`. If `duration_mins > 0`, execution is limited to that simulated duration. If `duration_mins <= 0`, no time limit is applied. |
| **Error handling** | None. |
| **Memory**      | No allocation. |

### Value Management

#### DDR-FFI-14: hippocrates_engine_set_value

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `int hippocrates_engine_set_value(EngineContext* ctx, const char* var_name, const char* json_val)` |
| **Purpose**     | Sends a variable value to the engine using the environment's current abstract time as the timestamp. |
| **Preconditions** | All three parameters are non-null. `var_name` identifies a defined variable. `json_val` is a JSON-encoded value compatible with the variable's type. |
| **Postconditions** | Returns 0 on success. The value is sent via `mpsc` channel as an `InputMessage` for the executor to process. The variable name is normalized (angle brackets stripped). |
| **Error handling** | Returns 1 for null pointers, invalid UTF-8, failed JSON parsing, failed input validation (precision/range), or channel send failure. |
| **Memory**      | No allocation returned to caller. |

Value parsing logic (`parse_runtime_value`):
- **Number type**: Parses as `f64`, or as `"N unit"` string producing `Quantity(N, unit)`.
- **DateTime type**: Parses date strings in `%Y-%m-%d %H:%M` or `%Y-%m-%d` format producing `RuntimeValue::Date`.
- **Other types**: Attempts `serde_json` deserialization, falls back to `RuntimeValue::String`.

#### DDR-FFI-15: hippocrates_engine_set_value_at

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `int hippocrates_engine_set_value_at(EngineContext* ctx, const char* var_name, const char* json_val, int64_t timestamp_ms)` |
| **Purpose**     | Same as `hippocrates_engine_set_value` but with an explicit timestamp in milliseconds. |
| **Preconditions** | Same as DDR-FFI-14. `timestamp_ms` must be a valid millisecond timestamp convertible to `NaiveDateTime`. |
| **Postconditions** | Returns 0 on success. The `InputMessage` carries the provided timestamp instead of `env.now`. |
| **Error handling** | Returns 1 for all conditions in DDR-FFI-14, plus invalid timestamp conversion. |
| **Memory**      | No allocation returned to caller. |

### Data Retrieval

#### DDR-FFI-16: hippocrates_get_periods

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `char* hippocrates_get_periods(EngineContext* ctx)` |
| **Purpose**     | Returns all period definitions from the environment as a JSON array. |
| **Preconditions** | `ctx` is a valid `EngineContext` with definitions loaded. |
| **Postconditions** | Returns a JSON array of `PeriodDef` objects. |
| **Error handling** | Returns null for null `ctx` or serialization failure. |
| **Memory**      | Caller must free via `hippocrates_free_string`. |

#### DDR-FFI-17: hippocrates_simulate_occurrences

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `char* hippocrates_simulate_occurrences(EngineContext* ctx, const char* period_name, int64_t start_ts, int duration_days)` |
| **Purpose**     | Calculates all occurrences of a named period within a date range, returning start/end pairs as ISO 8601 timestamps. |
| **Preconditions** | `ctx` and `period_name` are non-null. `period_name` references a defined period. `start_ts` is a valid millisecond timestamp. |
| **Postconditions** | Returns a JSON array of `{"start": "...", "end": "..."}` objects. |
| **Error handling** | Returns null for null pointers, invalid UTF-8, undefined period, or serialization failure. |
| **Memory**      | Caller must free via `hippocrates_free_string`. |

#### DDR-FFI-18: hippocrates_engine_get_values

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `char* hippocrates_engine_get_values(EngineContext* ctx, int64_t start_ts, int64_t end_ts)` |
| **Purpose**     | Returns all variable value snapshots within the specified time window `[start_ts, end_ts)`. |
| **Preconditions** | `ctx` is a valid `EngineContext`. Timestamps are milliseconds. `end_ts > start_ts`. |
| **Postconditions** | Returns a JSON array of `{"variable": "...", "display": "<...>", "value": "...", "timestamp": N}` objects sorted by timestamp then variable name. |
| **Error handling** | Returns null for null `ctx` or invalid timestamps. Returns `"[]"` for `end_ts <= start_ts`. |
| **Memory**      | Caller must free via `hippocrates_free_string`. |

### Time Management

#### DDR-FFI-19: hippocrates_engine_set_time

| Item            | Detail |
|-----------------|--------|
| **Signature**   | `void hippocrates_engine_set_time(EngineContext* ctx, int64_t timestamp_ms)` |
| **Purpose**     | Sets the abstract current time of the engine environment. |
| **Preconditions** | `ctx` is a valid `EngineContext`. `timestamp_ms` is a valid millisecond timestamp. |
| **Postconditions** | `env.now` is updated to the corresponding `NaiveDateTime`. Subsequent value operations and evaluations use this time. |
| **Error handling** | Silently does nothing if timestamp conversion fails. |
| **Memory**      | No allocation. |

---

## 3. Domain Model

All domain types are defined in `src/domain.rs`.

### DDR-DOM-01: RuntimeValue

`RuntimeValue` is the central value representation used throughout the engine at execution time.

| Variant            | Payload                | Description |
|--------------------|------------------------|-------------|
| `Number(f64)`      | Scalar number          | A dimensionless numeric value. |
| `Quantity(f64, Unit)` | Number with unit    | A numeric value with an associated measurement unit. |
| `String(String)`   | Text                   | A free-text string value. |
| `Boolean(bool)`    | True/false             | A boolean value. |
| `Enumeration(String)` | Named label         | An enumeration case label (e.g., `"Yes"`, `"No"`). |
| `List(Vec<RuntimeValue>)` | Collection      | An ordered list of values. |
| `Date(NaiveDateTime)` | Date/time           | A calendar date and time (timezone-naive). |
| `Void`             | None                   | Represents absence of a meaningful value. |
| `NotEnoughData`    | None                   | Signals insufficient historical data for a statistical calculation. |
| `Missing(String)`  | Variable name          | Indicates a value that has not been provided yet. |

Helper methods:
- `as_date() -> Option<NaiveDateTime>`: Extracts date if variant is `Date`.
- `as_number() -> Option<f64>`: Extracts numeric component from `Number` or `Quantity`.

### DDR-DOM-02: Unit

`Unit` represents measurement units with built-in conversion support.

| Category     | Variants |
|-------------|----------|
| Temperature | `Fahrenheit`, `Celsius` |
| Percentage  | `Percent` |
| Weight      | `Milligram`, `Gram`, `Kilogram`, `Pound`, `Ounce` |
| Length       | `Meter`, `Centimeter`, `Millimeter`, `Kilometer`, `Inch`, `Foot`, `Mile` |
| Volume       | `Liter`, `Milliliter`, `FluidOunce`, `Gallon` |
| Time         | `Year`, `Month`, `Week`, `Day`, `Hour`, `Minute`, `Second` |
| Pressure     | `MillimeterOfMercury` |
| Clinical     | `Bpm`, `MgPerDl`, `MmolPerL` |
| Custom       | `Custom(String)` |

The `convert(&self, value, target) -> Result<f64, String>` method supports same-category conversions (e.g., Celsius to Fahrenheit, Kilogram to Gram, MmolPerL to MgPerDl via glucose approximation factor 18.0182). Returns `Err` for cross-category conversions.

### DDR-DOM-03: EventType

`EventType` classifies audit log entries. Represented as `u8` for FFI transmission.

| Variant        | Discriminant | Description |
|----------------|:------------:|-------------|
| `Log`          | 0            | General execution log message. |
| `Message`      | 1            | Information/warning/urgent warning delivery. |
| `Question`     | 2            | A question was posed to the user. |
| `Answer`       | 3            | An answer was received from the user. |
| `Decision`     | 4            | An assessment/conditional decision was made. |
| `StateChange`  | 5            | A variable value changed. |
| `EventTrigger` | 6            | A scheduled or triggered event fired. |

### DDR-DOM-04: AskRequest

`AskRequest` is serialized to JSON and passed to the `AskCallback` when the engine needs user input.

| Field              | Type                       | Description |
|--------------------|----------------------------|-------------|
| `variable_name`    | `String`                   | Name of the variable being asked about. |
| `question_text`    | `String`                   | Human-readable question prompt. |
| `style`            | `QuestionStyle`            | UI style hint (see below). |
| `options`          | `Vec<String>`              | Available options for `Selection` style. |
| `range`            | `Option<(f64, f64)>`       | Numeric range constraint (min, max). |
| `date_time_range`  | `Option<(i64, i64)>`       | Date/time range in milliseconds. |
| `time_range`       | `Option<(String, String)>` | Time-of-day range as strings. |
| `date_only`        | `bool`                     | Whether only a date (not time) is requested. |
| `validation_mode`  | `Option<ValidationMode>`   | `Once` or `Twice` (double-entry verification). |
| `validation_timeout` | `Option<i64>`            | Timeout for validation in milliseconds. |
| `timestamp`        | `i64`                      | When the question was posed (ms). |
| `valid_after`      | `Option<i64>`              | Minimum timestamp for a valid cached answer (ms). |

`QuestionStyle` variants: `Text`, `Selection`, `Likert`, `VisualAnalogueScale { min, max, min_label, max_label }`, `Numeric`, `Date`, `Unknown`.

`ValidationMode` variants: `Once`, `Twice`.

### DDR-DOM-05: AuditEntry

`AuditEntry` records a single auditable event during plan execution.

| Field        | Type              | Description |
|-------------|-------------------|-------------|
| `timestamp` | `NaiveDateTime`   | When the event occurred. |
| `event_type`| `EventType`       | Classification of the event. |
| `details`   | `String`          | JSON payload or descriptive text. |
| `context`   | `Option<String>`  | Context name or rule name associated with the event. |

### DDR-DOM-06: ValueType

`ValueType` defines the kind of a value definition, determining parsing, validation, and UI behavior.

| Variant          | Description |
|------------------|-------------|
| `Number`         | Numeric value (requires unit and valid values range). |
| `Enumeration`    | Discrete labeled options. |
| `String`         | Free-text value. |
| `TimeIndication` | Time-of-day or weekday reference. |
| `DateTime`       | Calendar date and/or time. |
| `Period`         | Reference to a period definition. |
| `Plan`           | Reference to a plan definition. |
| `Drug`           | Reference to a drug definition. |
| `Addressee`      | Reference to an addressee definition. |
| `AddresseeGroup` | Reference to an addressee group definition. |

Supporting type: `ValueDefinition { name: String, value_type: ValueType }`.

Supporting type: `ValueInstance { value: RuntimeValue, timestamp: NaiveDateTime }` -- a timestamped value used in the history store.

Supporting type: `EngineError { message: String, line: usize, column: usize }` -- error type used throughout parsing and validation, implementing `Display` and `Error`.

---

## 4. Parser

### DDR-PARSER-01: PEG Grammar Structure

The grammar is defined in `src/grammar.pest` and processed by the `pest` parser generator. Top-level rules:

| Rule                  | Description |
|-----------------------|-------------|
| `file`                | Entry rule: `SOI ~ NEWLINE* ~ definition* ~ EOI` |
| `definition`          | Dispatches to one of: `unit_definition`, `drug_definition`, `addressee_definition`, `context_definition`, `plan_definition`, `period_definition`, `value_definition` |
| `value_definition`    | `identifier "is" value_type (":" INDENT value_property+ DEDENT | "."?)` |
| `period_definition`   | `identifier "is a period:" INDENT period_property+ DEDENT` |
| `plan_definition`     | `identifier "is a plan:" INDENT plan_block+ DEDENT` |
| `drug_definition`     | `identifier "is a drug:" INDENT (ingredients/dosage/admin/interaction)* DEDENT` |
| `addressee_definition`| `identifier ("is an addressee" | "is an addressee group") ":" INDENT ... DEDENT` |
| `context_definition`  | `"context:" INDENT context_item+ DEDENT` |
| `unit_definition`     | `identifier "is a unit:" INDENT unit_property+ DEDENT` |

Key expression and statement rules:
- `expression`: `term (infix_op term)*` with infix operators `+`, `-`, `*`, `/`.
- `term`: `date_diff | meaning_of_expr | quantity with relative modifier | statistical_func | quantity | datetime | date | time | string | identifier | "(" expression ")"`.
- `statement`: `timeframe_block | context_block | conditional | assignment | meaning_assignment | constraint | action`.
- `conditional`: `"assess" expression ":" INDENT assessment_case* DEDENT`.
- `identifier`: `angled_identifier` (e.g., `<Blood Pressure>`).

### DDR-PARSER-02: AST Node Hierarchy

Defined in `src/ast.rs`.

**Top-level:**
- `Plan { definitions: Vec<Definition> }`

**Definition variants:**
- `Value(ValueDef)` -- name, value_type, properties
- `Period(PeriodDef)` -- name, timeframes, line
- `Plan(PlanDef)` -- name, blocks
- `Drug(DrugDef)` -- name, properties
- `Addressee(AddresseeDef)` -- name, is_group, properties
- `Context(ContextDef)` -- items
- `Unit(UnitDef)` -- name, plurals, singulars, abbreviations

**PlanBlock variants:**
- `BeforePlan(Vec<Statement>)` -- executed immediately at plan start
- `Event(EventBlock)` -- named event with trigger and statements
- `Trigger(TriggerBlock)` -- anonymous trigger with statements

**Trigger variants:**
- `Periodic { interval, interval_unit, duration, offset, specific_day }` -- recurring execution
- `StartOf(String)` -- fires at the beginning of a period or plan
- `ChangeOf(String)` -- fires when a variable changes

**StatementKind variants:**
- `Assignment(Assignment)` -- `target = expression`
- `Action(Action)` -- message, question, listen, configure actions
- `Conditional(Conditional)` -- assess expression with cases
- `ContextBlock(ContextBlock)` -- scoped context with statements
- `EventProgression(String, Vec<AssessmentCase>)` -- progression tracking
- `Command(String)` -- simple named command
- `Constraint(Expression, String, RangeSelector)` -- value constraint
- `Timeframe(TimeframeBlock)` -- timeframe-scoped block
- `NoOp` -- empty statement

**Expression variants:**
- `Literal(Literal)` -- Number, String, Quantity, TimeOfDay, Date
- `Variable(String)` -- variable reference
- `MeaningOf(String)` -- meaning of a variable
- `Binary(Box<Expression>, String, Box<Expression>)` -- arithmetic operation
- `Statistical(StatisticalFunc)` -- aggregate functions
- `RelativeTime(f64, Unit, RelativeDirection)` -- "N units ago/from now"
- `DateDiff(Unit, Box<Expression>, Box<Expression>)` -- date difference
- `FunctionCall(String, Vec<Expression>)` -- generic function call
- `InterpolatedString(Vec<Expression>)` -- string with embedded expressions

**StatisticalFunc variants:**
- `CountOf(String, Option<Box<Expression>>)` -- count occurrences, optional filter
- `AverageOf(String, Box<Expression>)` -- average over period
- `MinOf(String)` -- minimum value
- `MaxOf(String)` -- maximum value
- `TrendOf(String)` -- trend calculation

**Action variants:**
- `ShowMessage { kind, parts, addressees, block }` -- display message (Information/Warning/UrgentWarning)
- `AskQuestion(String, Option<Vec<Statement>>)` -- prompt for input
- `SendInfo(String, Vec<Expression>)` -- send information
- `ListenFor(String)` -- passively wait for a value
- `StartPeriod` -- start a period
- `Configure(QuestionConfig)` -- question configuration
- `MessageExpiration(RangeSelector)` -- message expiry setting
- `ValidateAnswer(ValidationMode, Option<(f64, Unit)>)` -- answer validation

### DDR-PARSER-03: Indentation Handling

The Hippocrates language uses significant indentation (similar to Python). Since PEG grammars cannot natively handle indentation, a preprocessing step converts indentation to explicit tokens.

**Function:** `preprocess_indentation(input: &str) -> String`

**Algorithm:**
1. Maintain an `indent_stack` initialized with `[0]`.
2. For each line in the input:
   - Skip empty/whitespace-only lines.
   - Count leading spaces to determine the indentation level.
   - If the indentation increases beyond the current level, push the new level onto the stack and insert the Unicode marker character `'《'` (U+300A) into the output.
   - While the indentation decreases below the top of the stack, pop levels and insert `'》'` (U+300B).
   - Append the trimmed line content.
3. At end-of-input, emit remaining `'》'` markers for any open indentation levels.

The grammar rules `INDENT` and `DEDENT` match these marker characters:
- `INDENT = _{ "《" ~ "\n"? }`
- `DEDENT = _{ "》" ~ "\n"? }`

### DDR-PARSER-04: Parser Entry Point

**Function:** `parse_plan(input: &str) -> Result<Plan, EngineError>` in `src/parser.rs`.

**Flow:**
1. Call `preprocess_indentation(input)` to convert indentation to INDENT/DEDENT markers.
2. Invoke `HippocratesParser::parse(Rule::file, &processed)` (pest-derived parser).
3. Iterate over top-level `definition` pairs.
4. For each definition, dispatch to type-specific builders: `parse_value_def`, `parse_period_def`, `parse_plan_def`, `parse_drug_def`, `parse_addressee_def`, `parse_context_def`, `parse_unit_def`.
5. Collect all definitions into `Plan { definitions }`.
6. Validation is not performed here; it is the caller's responsibility (to support multi-error collection).

The parser struct `HippocratesParser` is derived via `#[derive(Parser)]` with `#[grammar = "grammar.pest"]`.

### DDR-PARSER-05: Ordinal and Bare-Unit Trigger Sugar

**Traces to:** DES-10, REQ-3.8-07, STKR-02

**Grammar change** (`src/grammar.pest`):
- Add `ordinal` rule: matches `"other" | "second" | "third" | "fourth" | "fifth" | "sixth" | "seventh" | "eighth" | "ninth" | "tenth"`.
- Add `bare_unit` rule: matches `"day" | "week" | "month" | "year" | "hour" | "minute" | "second"` (the time unit words without a leading quantity).
- Update `event_trigger` first periodic alternative to: `"every" ~ (ordinal ~ bare_unit | bare_unit | quantity ~ identifier?) ~ ("at" ~ time_literal)? ~ ("for" ~ quantity)?`.

**Parser change** (`src/parser.rs`):
- `parse_event_trigger` maps ordinals to their numeric value: other/second = 2, third = 3, fourth = 4, fifth = 5, sixth = 6, seventh = 7, eighth = 8, ninth = 9, tenth = 10.
- Bare units (no ordinal, no quantity) are desugared to interval = 1.0 with the matched unit.
- The AST `Trigger::Periodic` is unchanged -- ordinals and bare units are desugared to numeric interval and unit at parse time.

**Formatter change** (`src/formatter.rs`):
- Ordinals are NOT round-tripped; they desugar to `every 3 days` etc.
- Bare unit `every day` could be round-tripped but for simplicity is normalized to `every 1 day`.

---

## 5. Validator

### DDR-VAL-01: Validation Pipeline

Entry point: `validate_file(plan: &Plan) -> Result<(), Vec<EngineError>>` in `src/runtime/validator/mod.rs`.

**Pipeline order:**

1. **Build lookup structures:** Construct `defs_map` (HashMap of all definitions by name), `unit_map`, `valid_units` set, `enum_vars` set, `defined_values` set.
2. **Collect value ranges and precision:** For each numeric `ValueDef`, extract `Interval` ranges from valid values, compute precision info (decimal places), and determine expected units.
3. **Enumeration checks:** Verify that enumeration valid values use identifier syntax (angle brackets).
4. **Precision consistency:** Call `input_validation::precision_for_value` to verify all intervals in a definition use the same number of decimal places.
5. **Semantic checks** (`semantics` module):
   - `check_unit_definitions`: Verify custom units do not conflict with built-in units.
   - `check_drugs`: Verify drug ingredient units are defined.
   - `check_addressees`: Verify email format in contact info.
   - `check_value_definitions`: Verify Number types have units, Enumerations have valid values.
   - `check_timeframe_period_references`: Verify timeframe selectors reference defined periods.
6. **Calculation unit/precision checks:** Verify assignment units and precision match value definitions.
7. **Valid value overlap checks:** Detect overlapping numeric ranges or datetime ranges.
8. **Meaning coverage** (`coverage` module): Check that assessment cases cover the full valid range.
9. **Plan-level checks:** For each plan definition:
   - Statement semantic checks (`check_statement_semantics`): undefined variables, missing question properties.
   - Data flow analysis (`data_flow::analyze_block`): use-before-assignment detection.
10. **Return:** If errors are empty, return `Ok(())`. Otherwise return `Err(errors)`.

### DDR-VAL-02: Semantic Validation

Implemented in `src/runtime/validator/semantics.rs`.

| Check Function                      | Validations Performed |
|--------------------------------------|-----------------------|
| `check_unit_definitions`             | Custom unit names, plurals, singulars, and abbreviations must not conflict with built-in unit names. |
| `check_drugs`                        | Drug ingredient units must be either standard or user-defined units. |
| `check_addressees`                   | Addressee email addresses must contain `'@'`. |
| `check_value_definitions`            | Number and Enumeration types must have `valid values` property. Numbers must have a unit (explicit or implied from quantities in ranges). |
| `check_timeframe_period_references`  | Timeframe selectors referencing period names must point to defined periods. Checks definitions and plan statements recursively. |
| `check_statement_semantics`          | Validates assignments (undefined variable references), `ask` actions (variable must have question property and valid values if Number/Enumeration), `listen for` (must target defined variable), message parts (expression validation). Recurses into conditional branches. |
| `validate_expression`                | Checks that variables and meaning-of references point to defined names. Validates statistical function constraints (e.g., `count of` Enumeration requires a filter value; `trend of` not supported for Enumerations). |

### DDR-VAL-03: Interval Validation

Implemented in `src/runtime/validator/intervals.rs`.

**Core types:**
- `Interval { min: f64, max: f64 }` -- a numeric range clamped to non-negative values.
- `IntervalSet = Vec<Interval>` -- a collection of intervals.

**Key functions:**
- `merge_intervals(ranges)` -- sorts by `min` and merges overlapping intervals (epsilon tolerance 0.0001).
- `calculate_interval_set(expr, defined_ranges)` -- computes the possible interval set for an expression, supporting `+`, `-`, `*`, `/` operations on interval sets.
- `calculate_interval(expr, defined_ranges)` -- simplified single-interval calculation for expressions.
- `check_subtraction_safety(left, right, defined_ranges)` -- warns if a subtraction could produce a negative result.

### DDR-VAL-04: Data Flow Analysis

Implemented in `src/runtime/validator/data_flow.rs`.

**State:** `FlowState { initialized: HashSet<String> }` -- tracks which variables have been assigned or asked.

**Functions:**
- `analyze_block(stmts, parent_state, defs, errors)` -- processes a statement list, threading flow state forward.
- `analyze_statement(stmt, state, defs, errors)` -- per-statement analysis:
  - **Assignment:** Checks RHS expression uses, then marks LHS as initialized.
  - **AskQuestion:** Marks variable as initialized. Verifies question property exists.
  - **ListenFor:** Marks variable as initialized.
  - **ShowMessage/SendInfo:** Checks expression parts for uninitialized variables.
  - **Conditional:** Checks condition expression. Analyzes branches conservatively (branch results do not propagate to outer scope).
  - **ContextBlock:** Marks `data:` items as initialized, then analyzes inner statements.
  - **TimeframeBlock:** Analyzes inner statements in current flow state.
- `check_expression(expr, state, defs, line, errors)` -- reports use-before-assignment for `Variable` and `MeaningOf` references. Statistical functions are exempt (they operate on history).

### DDR-VAL-05: Coverage Analysis

Implemented in `src/runtime/validator/coverage.rs`.

**Numeric coverage** (`check_coverage`):
- Receives the valid ranges for a variable, the assessment cases, precision (decimal count), and reports gaps or overlaps.
- For discrete (decimal-precision) values: `check_discrete_coverage` steps through the value space checking each discrete step.
- For continuous values: `check_continuous_coverage` sweeps through sorted, clamped ranges detecting gaps and overlaps with epsilon tolerance.
- Skips analysis if any range bound is infinite.

**String/enumeration coverage** (`check_string_coverage`):
- Extracts string/identifier labels from assessment case selectors.
- Reports any required values not covered by any case.

### DDR-VAL-06: Error Reporting

All validation errors use `EngineError { message: String, line: usize, column: usize }`.

- Errors are collected into a `Vec<EngineError>` throughout the validation pipeline.
- The `line` field indicates the source line (1-based when available, 0 when line information is not tracked).
- The `column` field is typically 0 (column-level precision is not tracked in most validators).
- `EngineError` implements `Display` (formatted as `"Error at line N, col N: message"`) and `Error`.
- For FFI, errors are serialized to JSON via `serde`: `{"message": "...", "line": N, "column": N}`.

---

## 6. Runtime

### DDR-RT-01: Engine Struct

Defined in `src/runtime/mod.rs`.

```
pub struct Engine {
    pub env: Environment,
    pub mode: ExecutionMode,
}
```

The `Engine` struct provides a high-level API for Rust callers:
- `new()` -- creates with `RealTime` mode and empty environment.
- `set_mode(mode)` -- switches execution mode.
- `load_plan(plan)` -- delegates to `env.load_plan`.
- `execute(plan_name)` -- creates a temporary `Executor` and runs the plan.
- `set_value(name, val)` -- directly sets a value in the environment.

The FFI layer uses `EngineContext` instead, which adds `user_data`, `input_sender`, and `stop_signal` fields.

Helper functions:
- `normalize_identifier(name)` -- strips angle brackets from `<Name>` to produce `Name`.
- `format_identifier(name)` -- wraps bare names in angle brackets.

### DDR-RT-02: Environment

Defined in `src/runtime/environment.rs`.

```
pub struct Environment {
    pub values: HashMap<String, Vec<ValueInstance>>,
    pub definitions: HashMap<String, Definition>,
    pub now: NaiveDateTime,
    pub start_time: NaiveDateTime,
    pub output_log: Vec<String>,
    pub output_handler: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub context_stack: RwLock<Vec<EvaluationContext>>,
    pub unit_map: HashMap<String, Unit>,
    pub audit_log: Vec<AuditEntry>,
}
```

| Field            | Purpose |
|------------------|---------|
| `values`         | Variable history store. Each variable maps to a `Vec<ValueInstance>` ordered by timestamp. |
| `definitions`    | All loaded AST definitions indexed by name. |
| `now`            | Current abstract time (mutable, advanced by executor or set externally). |
| `start_time`     | When execution began. |
| `output_log`     | Plain-text log buffer. |
| `output_handler`  | Optional callback for log output. |
| `context_stack`  | Stack of `EvaluationContext` used by the evaluator for scoped timeframe and period resolution. Uses `RwLock` for interior mutability. |
| `unit_map`       | Maps custom unit names (including plurals/singulars/abbreviations) to canonical `Unit` values. |
| `audit_log`      | Chronological list of `AuditEntry` records. |

**EvaluationContext:** `{ timeframe: Option<RangeSelector>, period: Option<String> }` -- constructed from timeframe/context constraints to scope statistical queries and value lookups.

**Key methods:**
- `load_plan(plan)` -- iterates definitions, inserts into `definitions` map, initializes variables with default values at epoch, populates `unit_map` for custom units.
- `set_value(name, val)` / `set_value_at(name, val, timestamp)` -- appends to value history.
- `push_context(ctx)` / `pop_context()` / `active_context()` -- manage the evaluation context stack.
- `expected_unit_for_value(name)` -- determines the canonical unit for a numeric value from its definition.

### DDR-RT-03: Executor

Defined in `src/runtime/executor.rs`.

**Struct:**
```
pub struct Executor {
    pub on_step: Option<Box<dyn Fn(usize) + Send>>,
    pub on_log: Option<Box<dyn Fn(String, EventType, NaiveDateTime) + Send>>,
    pub on_ask: Option<Box<dyn Fn(AskRequest) + Send>>,
    pub on_message: Option<Box<dyn Fn(String, NaiveDateTime) + Send>>,
    pub mode: ExecutionMode,
    pub input_receiver: Option<mpsc::Receiver<InputMessage>>,
    pub stop_signal: Arc<AtomicBool>,
    next_event_time: Option<NaiveDateTime>,
    pending_inputs: Vec<InputMessage>,
}
```

**ScheduledEvent:** A time-stamped event in the priority queue.
```
struct ScheduledEvent {
    time: NaiveDateTime,
    kind: EventKind,
}
```
Implements reverse `Ord` for min-heap behavior in `BinaryHeap` (earliest event pops first).

**EventKind variants:**
- `Periodic { block, iteration, interval_secs, max_duration, time_of_day }` -- a repeating trigger block. When `time_of_day` is `Some(NaiveTime)`, both initial scheduling and rescheduling pin to that time (REQ-5-05).
- `PeriodicByPeriod { block }` -- a pre-scheduled event for a single occurrence of a named period within a duration window (REQ-3.8-06). All occurrences are enumerated at plan start via `Scheduler::occurrences_in_range`; no rescheduling.
- `StartOf { block, period_name }` -- fires at the start of a period.

**Execution flow (`execute_plan`):**
1. Clone definitions from environment.
2. Drain initial inputs.
3. Process `BeforePlan` blocks immediately.
4. Schedule `Periodic` triggers into the `BinaryHeap` with calculated first-run times. If `time_of_day` is set, pin first run to that time on the first eligible day. If the trigger references a named period (offset is a defined period name) and has a duration, use `Scheduler::occurrences_in_range` to enumerate all occurrences and schedule each as `PeriodicByPeriod`.
5. Schedule `StartOf` triggers by querying the scheduler for next period occurrences.
6. Enter event loop: pop events from the heap, advance `env.now`, execute associated statement blocks, reschedule `Periodic` events (pinning to time_of_day when set), execute `PeriodicByPeriod` events without rescheduling, drain input channel between events.
7. Exit when: heap is empty, stop signal is set, or simulation duration is exceeded.

**Input handling:**
- `drain_inputs(env)` -- receives messages from `mpsc` channel, normalizes variable names, sorts by timestamp, applies due inputs.
- `drain_inputs_with_triggers(env)` -- same but also processes change-of triggers.

### DDR-RT-04: Evaluator

Defined in `src/runtime/evaluator.rs`.

**Struct:** `pub struct Evaluator;` (stateless, all methods are associated functions).

**Entry point:** `Evaluator::evaluate(env: &Environment, expr: &Expression) -> RuntimeValue`

**Evaluation rules:**

| Expression Type     | Behavior |
|--------------------|----------|
| `Literal`          | Direct conversion: Number, String, Quantity (with unit map lookup), TimeOfDay, Date. |
| `Variable("now")`  | Returns `RuntimeValue::Date(env.now)`. |
| `Variable(name)`   | Checks for `Calculation` property first (derived value). If found, evaluates the calculation expression with appropriate context. Otherwise, looks up the latest value from `env.values`. |
| `MeaningOf(name)`  | Retrieves the current value, then resolves its meaning by matching against the `MeaningDef` assessment cases. Returns the matching meaning label as a `String`. |
| `Binary(l, op, r)` | Evaluates both sides. For arithmetic (`+`, `-`, `*`, `/`): performs unit-aware computation with automatic conversion when units differ but are compatible. |
| `Statistical`      | See statistical functions below. |
| `RelativeTime`     | Computes a date relative to `env.now`. |
| `DateDiff`         | Computes the difference between two dates in the specified unit. |
| `InterpolatedString` | Evaluates each part and concatenates string representations. |

**Statistical functions:**

| Function              | Implementation |
|-----------------------|----------------|
| `COUNT_OF(var, filter)` | Counts values in history, optionally filtered by a specific value or within active context timeframe/period. |
| `SUM_OF`              | Sums numeric values in history (implied by averaging logic). |
| `AVG_OF(var, period)` | Computes average of values within the specified period quantity. |
| `MIN_OF(var)`         | Returns minimum numeric value from history. |
| `MAX_OF(var)`         | Returns maximum numeric value from history. |
| `LAST_OF`             | Returns the most recent value (implicit via variable lookup). |
| `TREND_OF(var)`       | Computes trend direction from historical values (linear regression or simple comparison). |

**Meaning resolution:** When evaluating `MeaningOf(name)`, the evaluator:
1. Gets the current `RuntimeValue` for the variable.
2. Looks up the `MeaningDef` from the variable's properties.
3. Matches the value against assessment cases using `RangeSelector` matching.
4. Returns the first matching case's block result (typically a meaning label assignment).

### DDR-RT-05: Scheduler

Defined in `src/runtime/scheduler.rs`.

**Struct:** `pub struct Scheduler;` (stateless, all methods are associated functions).

**Key functions:**

- `next_occurrence(def: &Definition, now: NaiveDateTime) -> Option<(NaiveDateTime, NaiveDateTime)>`
  - For `Period` definitions: iterates timeframe groups, computes next occurrence for each, returns the earliest.
  - For `Value` definitions with `Timeframe` property: same logic using value timeframes.
  - Returns `(start, end)` tuple of the next occurrence.

- `occurrences_in_range(def, start, end) -> Vec<(NaiveDateTime, NaiveDateTime)>`
  - Iteratively calls `next_occurrence` starting from `start`, collecting all occurrences before `end`.
  - Advances cursor past each found occurrence to avoid infinite loops.

- `is_within_period(def, timestamp) -> bool`
  - Checks whether a given timestamp falls within any timeframe of a period or value definition.

Timeframe matching supports:
- Time-of-day ranges (e.g., `8:00 ... 20:00`).
- Weekday constraints (e.g., `Monday`).
- Date ranges.
- Named period references.

### DDR-RT-06: Session

Defined in `src/runtime/session.rs`.

**Struct:**
```
pub struct Session {
    common_definitions: Arc<Mutex<HashMap<String, (RuntimeValue, NaiveDateTime)>>>,
    executors: Arc<Mutex<Vec<mpsc::Sender<InputMessage>>>>,
    pending_requests: Arc<Mutex<HashSet<String>>>,
    on_ask: Arc<dyn Fn(AskRequest) + Send + Sync>,
    on_log: Arc<dyn Fn(String, EventType, NaiveDateTime) + Send + Sync>,
}
```

**Purpose:** Coordinates multiple concurrent plan executions that share state.

**Key methods:**

- `new(on_ask, on_log)` -- creates session with shared state containers.
- `provide_answer(variable, value)` -- updates common definitions, removes from pending requests, broadcasts the `InputMessage` to all registered executor channels. Disconnected channels are pruned.
- `run_script(source, plan_name)` -- spawns a new thread that:
  1. Parses the plan source.
  2. Creates a new `Environment` and `Executor`.
  3. Registers the executor's input channel.
  4. Bootstraps known values from `common_definitions`.
  5. Sets up ask callback with deduplication: if the variable is already known (and valid), sends the cached answer immediately; if already pending, suppresses duplicate asks.
  6. Executes the plan.

**Shared state coordination:**
- `common_definitions`: `Arc<Mutex<HashMap>>` storing the latest value and timestamp for each variable across all plans.
- `executors`: `Arc<Mutex<Vec<Sender>>>` for broadcasting answers to all running plans.
- `pending_requests`: `Arc<Mutex<HashSet>>` preventing duplicate question prompts.

### DDR-RT-07: Input Validation

Defined in `src/runtime/input_validation.rs`.

**Entry point:** `validate_input_value(definitions, variable, value) -> Result<(), String>`

**Validation logic:**
1. Look up the variable's definition. If not found or not a `Value` definition, accept (return `Ok`).
2. If the value type is not `Number`, accept.
3. Determine the precision (number of decimal places) from the valid values specification using `precision_for_value`.
4. If precision is determined, verify the input value matches using `value_has_precision`.
5. Reject with descriptive error if precision does not match.

**Precision extraction (`precision_for_value`):**
- Scans `ValidValues` properties for `Constraint` and `EventProgression` statements.
- Extracts decimal counts from `Number(f64, Option<usize>)` and `Quantity(f64, Unit, Option<usize>)` literals.
- Enforces that all intervals use the same precision. Returns `Err` if mismatched.

**Precision checking (`value_has_precision`):**
- Multiplies the value by `10^decimals`, rounds, and checks if the difference is less than `1e-9`.
- Non-numeric values pass validation.

### DDR-RT-08: Execution Modes

Defined in `src/runtime/executor.rs`.

```
pub enum ExecutionMode {
    RealTime,
    Simulation { speed_factor: Option<f64>, duration: Option<chrono::Duration> },
}
```

| Mode          | Behavior |
|---------------|----------|
| `RealTime`    | The executor waits for real time to pass between scheduled events. Input is received via the `mpsc` channel from external sources. The event loop blocks on channel receive with timeout until the next event time. |
| `Simulation`  | Events are processed as fast as possible without real-time delays. `speed_factor: None` means instant execution. `duration` optionally limits the total simulated time span. The executor advances `env.now` to each event's scheduled time without waiting. |

### DDR-RT-09: Time-of-Day Pinning and Period-Based Repetition

**Traces to:** DES-13, REQ-3.8-05, REQ-3.8-06, REQ-5-05

**AST change** (`src/ast.rs`):
- `Trigger::Periodic` gains `time_of_day: Option<String>` field (`#[serde(default)]`), storing a `"HH:MM"` literal when the `at` clause is present.

**Grammar change** (`src/grammar.pest`):
- Both `every` forms in `event_trigger` gain an optional `("at" ~ time_literal)?` clause between the quantity/identifier and the `for` clause.

**Parser change** (`src/parser.rs`):
- `parse_event_trigger` collects `Rule::time_literal` tokens into the new `time_of_day` field.

**Executor change** (`src/runtime/executor.rs`):

1. **Time-pinned scheduling (REQ-5-05):** When `time_of_day` is `Some`, the first run is scheduled at that time on the current day (or next day if already past). Rescheduling advances by `interval_secs` then pins to the target time on the resulting date.

2. **Period-based repetition (REQ-3.8-06):** When a `Trigger::Periodic` has `offset` referring to a defined period and `duration` is set, the executor uses `Scheduler::occurrences_in_range(def, start_time, start_time + duration)` to enumerate all occurrences and pre-schedules each as `EventKind::PeriodicByPeriod { block }`. These events execute once without rescheduling.

**Formatter change** (`src/formatter.rs`):
- Periodic trigger formatting emits `at HH:MM` when `time_of_day` is present.

### DDR-RT-10: After Plan Execution

**Traces to:** DES-13, REQ-3.7-10, STKR-36

**AST change** (`src/ast.rs`):
- `PlanBlock` enum gains an `AfterPlan(Vec<Statement>)` variant, representing the `after plan:` block.

**Grammar change** (`src/grammar.pest`):
- `plan_block` gains `after_plan_block` as an alternative: `plan_block = { before_plan_block | after_plan_block | trigger_block | event_block }`.
- `after_plan_block = { "after plan" ~ flexible_block }`.

**Parser change** (`src/parser.rs`):
- `parse_plan_block` handles `Rule::after_plan_block` by parsing the contained statements and producing `PlanBlock::AfterPlan(stmts)`.

**Executor change** (`src/runtime/executor.rs`):
- After the event loop in `execute_plan` exits (heap empty or stop signal), the executor iterates `plan_def.blocks` and executes any `PlanBlock::AfterPlan(stmts)` blocks exactly once via the standard statement execution path.

**Formatter change** (`src/formatter.rs`):
- `format_script` emits `after plan:` with proper indentation when encountering an `AfterPlan` block, mirroring the `before plan:` formatting logic.

---

## 7. Formatter

### DDR-FMT-01: format_script

Defined in `src/formatter.rs`.

**Signature:** `pub fn format_script(input: &str) -> Result<String, EngineError>`

**Purpose:** Pretty-prints a Hippocrates plan source string with normalized indentation and consistent formatting.

**Algorithm:**
1. `normalize_line_endings` -- converts `\r\n` and `\r` to `\n`.
2. `insert_statement_newlines` -- ensures each statement-ending `.` is followed by a newline (unless inside a string, comment, or part of a range `...`). Handles decimal numbers (`.` between digits) and ellipsis markers.
3. `preprocess_indentation` -- reuses the parser's indentation preprocessor to insert INDENT/DEDENT markers.
4. Parse with `HippocratesParser::parse(Rule::file, ...)`.
5. Walk the pest parse tree and emit formatted output using recursive `format_*` functions that track indentation depth.
6. Each `write_line` call emits `indent * 4` spaces followed by trimmed content and a newline.
7. The `strip_markers` helper removes the `'《'` and `'》'` Unicode markers from raw pair text.
8. The `ensure_block_header` helper appends `:` to block headers if not already present.

**Definition-specific formatters:** `format_value_definition`, `format_period_definition`, `format_plan_definition`, `format_drug_definition`, `format_addressee_definition`, `format_context_definition`, `format_unit_definition` -- each handles the structure-specific formatting of its definition type, delegating to shared helpers for properties, statements, and assessment cases.

---

## 8. Traceability

| DDR ID Range    | System Design Reference | Component |
|-----------------|------------------------|-----------|
| DDR-FFI-01..19  | DES-18                 | FFI Layer |
| DDR-DOM-01..06  | DES-15, DES-11         | Environment, AST |
| DDR-PARSER-01..05 | DES-10               | Parser |
| DDR-VAL-01..06  | DES-12                 | Validator |
| DDR-RT-01       | DES-13                 | Executor |
| DDR-RT-02       | DES-15                 | Environment |
| DDR-RT-03       | DES-13                 | Executor |
| DDR-RT-04       | DES-16                 | Evaluator |
| DDR-RT-05       | DES-14                 | Scheduler |
| DDR-RT-06       | DES-17                 | Session |
| DDR-RT-07       | DES-15                 | Environment (input validation) |
| DDR-RT-08       | DES-13                 | Executor (modes) |
| DDR-RT-09       | DES-13                 | Executor (time-of-day pinning, period repetition) |
| DDR-RT-10       | DES-13                 | Executor (after plan execution) |
| DDR-FMT-01      | DES-19                 | Formatter |

---

## Revision History

| Version | Date       | Author | Changes |
|---------|------------|--------|---------|
| 1.0     | 2026-03-20 | --     | Initial draft. Full module inventory, C-FFI interface (DDR-FFI-01 through DDR-FFI-19), domain model (DDR-DOM-01 through DDR-DOM-06), parser (DDR-PARSER-01 through DDR-PARSER-04), validator (DDR-VAL-01 through DDR-VAL-06), runtime (DDR-RT-01 through DDR-RT-08), formatter (DDR-FMT-01), and traceability matrix. |
| 1.1     | 2026-03-23 | V-Model | Added DDR-RT-09 (time-of-day pinning, period repetition). Updated DDR-RT-03 EventKind variants and execution flow. |
| 1.2     | 2026-03-23 | V-Model | Added DDR-RT-10 (after plan execution). PlanBlock gains AfterPlan variant; executor runs AfterPlan blocks after event loop exit. |
| 1.3     | 2026-03-23 | V-Model | Added DDR-PARSER-05 (ordinal and bare-unit trigger sugar). Grammar gains `ordinal` and `bare_unit` rules; parser desugars to numeric intervals at parse time. |
