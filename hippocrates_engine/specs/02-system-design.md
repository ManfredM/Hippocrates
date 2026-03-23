# System Design (DES)

**Document ID:** DES
**Version:** 1.0
**Status:** Draft

This document describes the as-built system design of the Hippocrates engine. It reflects the actual implementation, which diverges from the original `runtime_architecture.md` planning document in several significant ways (see Section 7).

---

## 1. Design Overview

### DES-01 — Language Selection: Rust

The engine is implemented in Rust (Edition 2024). Rust was selected for:
- **Memory safety** without garbage collection, critical for a medical runtime.
- **C-FFI compatibility** via `extern "C"` functions, enabling embedding in iOS (Swift) and other platforms.
- **No runtime overhead** — no VM, no GC pauses, deterministic execution.

### DES-02 — Dual Crate Output

The crate produces two library types (configured in `Cargo.toml`):
- `rlib` — standard Rust library for use by other Rust crates and tests.
- `staticlib` — C-compatible static library (`libhippocrates_engine.a`) for linking into host applications.

### DES-03 — C-FFI Boundary

All platform integration occurs through a C-compatible FFI layer (`ffi.rs`). The engine exposes opaque pointer handles, callback registration, and JSON-serialized data exchange. Host applications never access Rust internals directly.

### Design Approach

The original `runtime_architecture.md` described a **bytecode compiler + virtual machine** architecture with **SQLite** for persistent state storage. The actual implementation uses:

- **AST-walking interpretation** — the parser produces an AST that the executor traverses directly. There is no bytecode compilation step and no virtual machine.
- **In-memory state** — all value history, definitions, and context are held in `HashMap`-based structures. There is no SQLite dependency and no on-disk persistence within the engine.

This simplification reduces complexity while preserving correctness. Persistence, if needed, is delegated to the host application via callbacks and the JSON-based value query API.

---

## 2. Component Architecture

```mermaid
graph TD
    Host[Host Application - iOS/Swift]
    FFI[FFI Layer - ffi.rs]

    subgraph "hippocrates_engine"
        Parser[Parser - Pest PEG]
        AST[AST - Typed Tree]
        Validator[Validator - Multi-Layer]
        Env[Environment - State Store]
        Exec[Executor - Event Loop]
        Sched[Scheduler - Occurrence Calc]
        Eval[Evaluator - Expressions]
        Fmt[Formatter - Pretty Printer]
        Session[Session - Multi-Plan Coord]
    end

    Host <-->|C API + Callbacks| FFI
    FFI --> Parser
    Parser --> AST
    AST --> Validator
    AST --> Env
    Env --> Exec
    Exec --> Eval
    Exec --> Sched
    Exec -->|Callbacks| FFI
    Session -->|Spawns| Exec
    Fmt --> Parser
```

### DES-10 — Parser

The parser uses **Pest** (PEG parser generator). A `grammar.pest` file defines the Hippocrates language grammar. The parser module (`parser.rs`) transforms source text into the typed AST. Error messages include line and column numbers. An indentation preprocessor normalizes whitespace before parsing.

### DES-11 — AST Representation

The AST (`ast.rs`) is a typed tree of Rust structs and enums representing the full language: `Plan`, `Definition` (Value, Drug, Period, Unit, Addressee, Context, Plan), `Statement`, `Expression`, `Literal`, `Property`, `Action`, `Block`, `Trigger`, and `PlanBlock` variants. The AST is serializable via `serde` for JSON export through the FFI.

### DES-12 — Multi-Layer Validator

The validator (`runtime/validator/mod.rs`) performs static analysis on the parsed AST before execution. It is organized into four sub-modules:

- **`semantics`** — type checking, definition consistency, unit compatibility.
- **`intervals`** — numeric range analysis, gap/overlap detection in assessment cases.
- **`data_flow`** — use-before-assignment detection, dependency analysis.
- **`coverage`** — exhaustive case coverage verification for assessments and meaning models.

Validation produces a `Vec<EngineError>` with source locations. The FFI exposes validation independently from execution (`hippocrates_validate_file`).

### DES-13 — Runtime Executor

The executor (`runtime/executor.rs`) is an event-driven engine built on a `BinaryHeap<ScheduledEvent>` (min-heap by time). It supports three event kinds:

- **`Periodic`** — recurring triggers with interval, iteration counter, optional max duration, and optional time-of-day pinning (REQ-3.8-05, REQ-5-05).
- **`PeriodicByPeriod`** — pre-scheduled events for each occurrence of a named period within a duration window (REQ-3.8-06). All occurrences are enumerated up front via the Scheduler.
- **`StartOf`** — period-triggered events scheduled via the Scheduler.

The event loop pops events in chronological order, advances environment time, drains pending inputs, executes statement blocks via AST-walking, and reschedules recurring events. A sliding 30-day window is used for period-based event scheduling. When a time-of-day is specified, both initial scheduling and rescheduling pin to the target time. After the event loop exits (all triggers exhausted or simulation time limit reached), the executor iterates plan blocks looking for `AfterPlan` blocks and executes their statements exactly once.

### DES-14 — Scheduler

The scheduler (`runtime/scheduler.rs`) calculates period occurrences. It provides:
- `next_occurrence(def, now)` — finds the next start/end time for a period definition.
- `occurrences_in_range(def, start, end)` — enumerates all occurrences within a time range.

It handles timeframe groups with day-of-week, time-of-day, and recurrence patterns.

### DES-15 — Environment

The environment (`runtime/environment.rs`) is the central state container:
- **`values: HashMap<String, Vec<ValueInstance>>`** — append-only value history keyed by variable name.
- **`definitions: HashMap<String, Definition>`** — all loaded definitions from the AST.
- **`now: NaiveDateTime`** — current abstract time (advanced by executor or set externally).
- **`start_time: NaiveDateTime`** — plan start timestamp.
- **`context_stack: RwLock<Vec<EvaluationContext>>`** — evaluation context stack for scoped timeframe/period resolution.
- **`unit_map: HashMap<String, Unit>`** — canonical unit lookup.
- **`audit_log: Vec<AuditEntry>`** — structured audit trail of engine events.
- **`output_handler`** — optional callback for debug/log output.

### DES-16 — Evaluator

The evaluator (`runtime/evaluator.rs`) resolves expressions to `RuntimeValue` results. It handles:
- Literal evaluation (numbers, strings, quantities, dates, times).
- Variable lookup (including the special `now` variable).
- Derived value calculation (evaluating `Property::Calculation` rules).
- Statistical functions over value history within timeframes.
- Meaning resolution (mapping numeric values to clinical labels via meaning models).
- Context-aware evaluation using the environment's context stack.

### DES-17 — Session

The session (`runtime/session.rs`) coordinates multiple concurrent plan executions:
- Spawns each plan in a separate thread via `thread::spawn`.
- Maintains **shared state** through `Arc<Mutex<...>>` wrappers:
  - `common_definitions` — shared variable values across plans.
  - `executors` — `mpsc::Sender` handles for broadcasting inputs to all running plans.
  - `pending_requests` — de-duplicated `Ask` requests across plans.
- `provide_answer()` broadcasts values to all executors and updates shared state.
- `run_script()` bootstraps a new executor thread with current shared state.

### DES-18 — FFI Layer

The FFI layer (`ffi.rs`) provides the C-compatible API:
- **`EngineContext`** — opaque struct holding `Environment`, `Executor`, `mpsc::Sender`, `stop_signal`, and `user_data` pointer.
- **Lifecycle**: `hippocrates_engine_new` / `hippocrates_engine_free` with `Box::into_raw` / `Box::from_raw` ownership transfer.
- **Callbacks**: Four callback types registered with `user_data` pointer for context:
  - `LineCallback` — current execution line number.
  - `LogCallback` — structured log events with type, message, and timestamp.
  - `AskCallback` — question/input requests serialized as JSON.
  - `MessageCallback` — message delivery payloads with timestamp.
- **Data exchange**: All structured data crosses the FFI boundary as JSON strings via `CString`. The host must call `hippocrates_free_string` to release returned strings.
- **`SendPtr`** wrapper enables sending raw `user_data` pointers across thread boundaries.

### DES-19 — Formatter

The formatter (`formatter.rs`) provides `format_script()` for pretty-printing Hippocrates source code. It re-parses the input using the Pest parser and emits consistently indented, normalized output. Used by editor tooling for code formatting.

---

## 3. Technology Stack

### DES-20 — Pest Parser (2.8.5 / pest_derive 2.8.5)

PEG (Parsing Expression Grammar) parser generator. The grammar is defined in `grammar.pest` and compiled at build time via `pest_derive`. Chosen for its strong error reporting and declarative grammar definition.

### DES-21 — Chrono (0.4.43, with `serde` feature)

Date and time library used throughout the engine for `NaiveDateTime`-based time arithmetic, period scheduling, timestamp handling, and RFC 3339 serialization. All internal time is naive (no timezone), with UTC conversion at the FFI boundary.

### DES-22 — Serde (1.0.228) / Serde JSON (1.0.149)

Serialization framework used for:
- AST serialization to JSON for FFI export (`hippocrates_parse_json`).
- Callback payload serialization (ask requests, audit log, value snapshots).
- Runtime value deserialization from JSON input (`hippocrates_engine_set_value`).
- Error serialization with `{message, line, column}` structure.

### DES-23 — Thiserror (2.0.17)

Derive macro for `std::error::Error` implementations. Used for `EngineError` and other error types to provide structured error reporting with source location.

### DES-24 — Libc (0.2.180)

Provides C-compatible type definitions (`c_char`, `c_int`, `c_void`) used in the FFI layer for cross-language function signatures.

### DES-25 — Rand (0.9.2)

Listed as a dependency but not actively used in production code paths at this time. Likely reserved for future use (e.g., randomized test data generation or stochastic simulation scenarios).

### DES-26 — In-Memory State Management

All state (value history, definitions, context, audit log) is stored in-memory using Rust `HashMap` and `Vec` collections. **SQLite is not used**, contrary to the original architecture document's recommendation. Persistence is the responsibility of the host application, which can query value history via `hippocrates_engine_get_values` and reconstruct state via `hippocrates_engine_set_value_at`.

---

## 4. Data Flow

The engine processes data through a linear pipeline with an event-driven execution loop:

```
Source Text (.hipp)
    |
    v
[Parser] -- Pest PEG grammar --> [AST] -- typed tree of definitions, plans, statements
    |
    v
[Validator] -- static analysis (semantics, intervals, data_flow, coverage)
    |
    v
[Environment.load_plan()] -- definitions and plans loaded into HashMap state
    |
    v
[Executor.execute_plan()] -- event loop:
    |   1. Build initial ScheduledEvent heap from plan triggers
    |   2. Pop next event (min-heap by time)
    |   3. Advance abstract time
    |   4. Drain input channel (mpsc receiver) for external values
    |   5. Execute statement block (AST-walking)
    |       - Evaluator resolves expressions
    |       - Environment stores values
    |       - Scheduler calculates period occurrences
    |   6. Fire callbacks (Line, Log, Ask, Message)
    |   7. Reschedule periodic events
    |   8. Repeat until heap empty or stop signal
    v
[Callbacks] --> Host Application
```

For the FFI path, the host interacts asynchronously: it calls `hippocrates_engine_execute` (which blocks on the event loop), and injects values via `hippocrates_engine_set_value` which sends messages through an `mpsc` channel that the executor drains each tick.

---

## 5. Embedding Strategy

### DES-30 — Initialization and Lifecycle

The host creates an engine instance via `hippocrates_engine_new(user_data)`, which returns an opaque `EngineContext*` pointer. This pointer is passed to all subsequent API calls. The host must call `hippocrates_engine_free(ctx)` to release the engine. Internally, the context is heap-allocated via `Box` and ownership is transferred across the FFI boundary using `Box::into_raw` / `Box::from_raw`.

### DES-31 — Callback Registration Model

Four callback types are supported, each receiving a `user_data` pointer for host-side context:

| Callback | Signature | Purpose |
|---|---|---|
| `LineCallback` | `(int line, void* user_data)` | Current execution line for debugger/stepper |
| `LogCallback` | `(char* msg, uint8_t type, int64_t ts_ms, void* user_data)` | Structured event log with type enum and timestamp |
| `AskCallback` | `(char* json, void* user_data)` | Input request (question) serialized as JSON |
| `MessageCallback` | `(char* json, int64_t ts_ms, void* user_data)` | Message delivery payload with timestamp |

Callbacks are registered via `hippocrates_engine_set_callbacks` and `hippocrates_engine_set_message_callback`. A `SendPtr` wrapper makes raw `user_data` pointers safe to move across thread boundaries.

### DES-32 — JSON Serialization for Data Exchange

All structured data crosses the FFI boundary as null-terminated UTF-8 JSON strings:
- Parse results: `{"Ok": <ast>}` or `{"Err": {"message": "...", "line": N, "column": N}}`.
- Value input: JSON-encoded `RuntimeValue` via `hippocrates_engine_set_value`.
- Query results: JSON arrays from `hippocrates_get_periods`, `hippocrates_simulate_occurrences`, `hippocrates_engine_get_values`.
- Audit log: JSON array from `hippocrates_get_audit_log`.

### DES-33 — Memory Management

Strings returned from the engine are allocated via `CString::into_raw()`. The host **must** call `hippocrates_free_string()` to release them, which reconstructs the `CString` via `CString::from_raw()` and drops it. Failure to free returned strings causes memory leaks.

### DES-34 — iOS Integration

The engine is compiled as a static library (`libhippocrates_engine.a`). A C bridging header (`hippocrates_engine.h`) declares all public functions and types. Swift code calls the C API directly using Swift's C interoperability. The `EngineContext` is treated as an opaque pointer managed by a Swift wrapper class.

---

## 6. Execution Model

### DES-40 — Real-Time Execution Mode

In `ExecutionMode::RealTime`, the executor sleeps (`thread::sleep`) for the actual duration between events. Time advances in lockstep with wall-clock time. This is the default mode for production execution on a patient's device.

### DES-41 — Simulation Mode

In `ExecutionMode::Simulation { speed_factor, duration }`:
- **`speed_factor: None`** — instant execution; events fire without delay (timelapse mode). Used for testing and visualization.
- **`speed_factor: Some(f64)`** — accelerated execution; sleep durations are divided by the factor (e.g., factor 10.0 runs 10x faster).
- **`duration: Option<Duration>`** — optional time limit; the executor stops when abstract time exceeds start + duration.

Simulation mode is enabled via `hippocrates_engine_enable_simulation` (FFI) or `executor.set_mode()` (Rust API).

### DES-42 — Input Channel

External values are injected via an `mpsc::channel<InputMessage>`:
- The FFI layer holds the `Sender` in `EngineContext`.
- The executor holds the `Receiver` and drains it each tick via `drain_inputs()`.
- Each `InputMessage` carries: variable name, `RuntimeValue`, and timestamp.
- Input validation (`input_validation` module) checks values against definitions before sending.
- In `Session` mode, inputs are broadcast to all running executors.

### DES-43 — Stop Signal

Graceful termination uses `Arc<AtomicBool>`:
- The FFI layer exposes `hippocrates_engine_stop()` which sets the flag to `true`.
- The executor checks the flag each iteration of the event loop.
- When set, the executor breaks out of the loop and returns control.

---

## 7. Design Divergence Note

The original planning document (`specification/runtime_architecture.md`) described an architecture that differs from the actual implementation in several ways:

| Aspect | Planned Design | Actual Implementation |
|---|---|---|
| **Execution model** | Bytecode compiler + virtual machine | AST-walking interpreter (no bytecode, no VM) |
| **State storage** | SQLite (recommended) for transactional persistence | In-memory `HashMap`-based state; no persistence layer |
| **State persistence** | Automatic save to disk after every write | No built-in persistence; host queries values via API |
| **Event queue** | Persisted queue with missed-event recovery | In-memory `BinaryHeap`; no persistence or recovery |
| **API style** | `hipp_init(storage_path)`, `hipp_tick()` polling model | `hippocrates_engine_new(user_data)`, blocking event loop with callbacks |
| **Callback model** | `MsgCallback(text, color)`, `QuestionCallback(id, text, type)` | `LogCallback(msg, type, timestamp, user_data)`, `AskCallback(json, user_data)` |
| **Multi-plan support** | Not described | `Session` struct with thread-per-plan and shared state |
| **Validation** | `hipp_validate_plan` returning single error | Multi-layer validator returning `Vec<EngineError>` |
| **Android support** | JNI with dynamic library (`.so`) | Not yet implemented; iOS static library only |
| **Code formatting** | Not described | `Formatter` module for pretty-printing |

These divergences reflect pragmatic design decisions made during implementation. The AST-walking approach is simpler to develop and debug. The in-memory state model delegates persistence responsibility to the host, which already manages its own data layer (e.g., Core Data on iOS). The callback-driven execution model is more natural for mobile integration than a polling/tick approach.

---

## 8. Traceability

| DES ID | Description | Traces to STKR |
|---|---|---|
| DES-01 | Rust language selection | STKR-03, STKR-14 |
| DES-02 | Dual crate output (rlib + staticlib) | STKR-03 |
| DES-03 | C-FFI boundary | STKR-03, STKR-14 |
| DES-10 | Pest PEG parser | STKR-01, STKR-02 |
| DES-11 | AST representation | STKR-01 |
| DES-12 | Multi-layer validator | STKR-04, STKR-10, STKR-30, STKR-31, STKR-32, STKR-33, STKR-34, STKR-35 |
| DES-13 | Runtime executor | STKR-01, STKR-05, STKR-36 |
| DES-14 | Scheduler | STKR-05 |
| DES-15 | Environment (state store) | STKR-15, STKR-16, STKR-19 |
| DES-16 | Evaluator | STKR-01, STKR-12 |
| DES-17 | Session (multi-plan) | STKR-01, STKR-05 |
| DES-18 | FFI layer | STKR-03 |
| DES-19 | Formatter | STKR-02 |
| DES-20 | Pest dependency | STKR-01 |
| DES-21 | Chrono dependency | STKR-05, STKR-18 |
| DES-22 | Serde/JSON dependency | STKR-03 |
| DES-23 | Thiserror dependency | STKR-04 |
| DES-24 | Libc dependency | STKR-03 |
| DES-25 | Rand dependency | — |
| DES-26 | In-memory state (no SQLite) | STKR-16, STKR-19 |
| DES-30 | Initialization and lifecycle | STKR-03 |
| DES-31 | Callback registration | STKR-03, STKR-05 |
| DES-32 | JSON data exchange | STKR-03 |
| DES-33 | Memory management | STKR-03 |
| DES-34 | iOS integration | STKR-03 |
| DES-40 | Real-time execution mode | STKR-05 |
| DES-41 | Simulation mode | STKR-06 |
| DES-42 | Input channel (mpsc) | STKR-05, STKR-15 |
| DES-43 | Stop signal | STKR-05 |

---

## Revision History

| Version | Date | Changes |
|---|---|---|
| 1.0 | 2026-03-20 | Initial system design document reflecting actual implementation |
| 1.1 | 2026-03-23 | Updated DES-13: added PeriodicByPeriod event kind and time-of-day pinning |
| 1.2 | 2026-03-23 | Updated DES-13: added AfterPlan block execution after event loop exit. Added STKR-36 traceability. |
