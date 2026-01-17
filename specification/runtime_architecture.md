# Hippocrates Runtime Architecture (Rust)

## 1. Overview

The Hippocrates **Engine** is a portable, high-performance runtime designed to execute Hippocrates Language scripts. It is built in **Rust** to ensure memory safety, concurrency, and easy embedding into mobile applications (iOS/Android) via a C-Foreign Function Interface (FFI).

## 2. Architecture Diagram

```mermaid
graph TD
    ClientApp[Mobile App (iOS/Swift, Android/Kotlin)]
    FFI[C-FFI Layer (extern "C")]
    Engine[Hippocrates Engine (Rust)]
    
    subgraph "Engine Components"
        Parser[Parser & AST Builder]
        Compiler[Bytecode Compiler]
        VM[Virtual Machine / Interpreter]
        Store[State Store (Values & History)]
        Scheduler[Event Scheduler]
    end
    
    ClientApp <-->|Calls API| FFI
    FFI <-->|Control| VM
    FFI <-->|Callbacks| ClientApp
    
    VM -->|Reads/Writes| Store
    VM -->|Registers Events| Scheduler
    Scheduler -->|Triggers| VM
```

## 3. Technology Stack

* **Language**: Rust (Edition 2021 or later).
* **Parsing**: `pest` (PEG parser) or `chumsky` for robust error recovery and friendly syntax errors.
* **Serialization**: `serde` + `bincode` for fast state persistence; `serde_json` for interop.
* **Data Storage**:
  * *Option A (Lightweight)*: In-memory `HashMap` with asynchronous persistence to a file (JSON/Bincode).
  * *Option B (Robust)*: SQLite via `rusqlite` (bundled) for transactional guarantees suitable for medical data history. **Recommendation: SQLite.**
* **Time Handling**: `chrono` crate for timezone-aware arithmetic.

## 4. Component Details

### 4.1. The Parser

The parser ensures the script follows the Formal Specification.

* **Input**: `.hipp` text files.
* **Output**: Abstract Syntax Tree (AST).
* **Error Handling**: Must provide "physician-readable" error messages (e.g., "I didn't understand the unit used for body weight on line 42").

### 4.2. State Store (The "Chart")

The heart of Hippocrates is the patient's data history.

* **Structure**:
  * `ValueID` (Hash of name).
  * `Timestamp` (UTC).
  * `Data` (Magnitude + Unit).
  * `Confidence` (u8).
  * `Source` (Manual/Device).
* **Persistence**: Automatically saved to disk after every write. This ensures that if the mobile app is killed by the OS, state is preserved.

### 4.3. Event Scheduler

Handles temporal logic (`every 2 weeks`, `30 minutes after event`).

* **Persisted Queue**: Pending events are serialized. On startup, the engine checks for missed events (e.g., phone was off during a scheduled dose) and decides whether to *Skip*, *Execute Immediately*, or *Queue*.

## 5. Embedding API (C-FFI)

The engine exposes a minimalistic C-Interface.

### 5.1. Initialization

```c
typedef struct HippocratesEngine HippocratesEngine;

// Initialize a new engine instance with a path to the DB file
HippocratesEngine* hipp_init(const char* storage_path);

// Free the engine
void hipp_free(HippocratesEngine* engine);
```

### 5.2. Plan Management

```c
// Load and compile a script
int hipp_load_plan(HippocratesEngine* engine, const char* script_content);

// Validate a script without loading (Dry Run)
int hipp_validate_plan(const char* script_content, char** error_out);
```

### 5.3. Interaction

```c
// Update a value (e.g., from HealthKit)
// returns 1 if successful, 0 if validation failed
int hipp_set_value(HippocratesEngine* engine, const char* name, double value, const char* unit);

// Answer a pending question
int hipp_answer_question(HippocratesEngine* engine, const char* question_id, const char* answer);

// Tick the engine (run event loop for a step)
// returns >0 if actions need to be processed
int hipp_tick(HippocratesEngine* engine);
```

### 5.4. Callbacks (The Bridge)

The host app registers callbacks to handle side-effects.

```c
typedef void (*MsgCallback)(const char* text, const char* color);
typedef void (*QuestionCallback)(const char* id, const char* text, const char* type);

void hipp_register_message_handler(HippocratesEngine* engine, MsgCallback cb);
void hipp_register_question_handler(HippocratesEngine* engine, QuestionCallback cb);
```

## 6. Mobile Implementation Strategy

### 6.1. iOS (Swift)

* Create a `Hippocrates.xcframework` bundling the static Rust library (`libhippocrates.a`) and C headers.
* Use Swift's **C Interoperability** to wrap `HippocratesEngine*` in a safe Swift class `HippocratesRuntime`.
* Swift class manages the pointer lifecycle and converts `String` <-> `const char*`.

### 6.2. Android (Kotlin/Java)

* Compile Rust as a dynamic library (`libhippocrates.so`) using `cargo-ndk`.
* Use **JNI (Java Native Interface)** to bridge between JVM and Rust.
* Expose a Kotlin class `HippocratesRuntime` that loads the `.so` and calls native methods.
