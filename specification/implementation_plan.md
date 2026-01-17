# Implementation Plan - Hippocrates Language Specification & Runtime

## Goal

Create a formal specification for the Hippocrates Language and a proposal for a Rust-based Runtime environment suitable for embedding in iOS and Android applications.

## User Review Required

- **Language Formalization**: The original document is informal and contains "ToDo" notes. The formal specification will standardize these into a definitive grammar (EBNF-like) and semantic rules.
- **Runtime Choice**: Confirm Rust as the implementation language (as requested/favored).
- **API Design**: The embedding API will be designed as a C-FFI (Foreign Function Interface) to ensure broad compatibility with Swift (iOS) and Kotlin/Java (Android).

## Proposed Changes

### Documentation Artifacts

I will create two major artifacts in the `artifacts/` directory (or directly as files if preferred for easy download).

#### [NEW] [hippocrates_specification.md](file:///Users/manfred/.gemini/antigravity/brain/fc139430-2cad-427a-85ac-cc8964d7e4b1/hippocrates_specification.md)

A formal definition of the language.

- **Syntax**: Formal grammar (EBNF) for Plans, Paragraphs, Values, Events, Periods, and Actions.
- **Semantics**: Detailed rules for execution, context handling, variable resolution, and state management (History, Confidence, Units).
- **Standard Library**: Definition of built-in types (Time, Units, Medical defaults).

#### [NEW] [runtime_architecture.md](file:///Users/manfred/.gemini/antigravity/brain/fc139430-2cad-427a-85ac-cc8964d7e4b1/runtime_architecture.md)

A technical proposal for the `Hippocrates Engine`.

- **Core Architecture**: Rust-based engine.
  - **Parser**: Nom or Pest based parser.
  - **Interpreter**: AST-walking interpreter or Bytecode VM (likely AST for flexibility first).
  - **State Store**: Persistent storage handling (SQLite or embedded KV store).
  - **Scheduler**: Event loop for temporal events and value listeners.
- **Mobile Integration (Embedding)**:
  - **C-API Layer**: `extern "C"` functions for initialization, plan loading, value updates, and callback registration.
  - **Platform Bindings**:
    - **iOS**: Swift wrapper around the C-header.
    - **Android**: JNI (Java Native Interface) wrapper for Kotlin.

## Verification Plan

### Manual Verification

- **Grammar Check**: I will verify that the examples found in `hippocrates.md` can be theoretically parsed by the proposed EBNF grammar.
- **API Walkthrough**: I will write pseudo-code examples showing how an iOS app would initialize the engine and handle a "Ask Question" event.
