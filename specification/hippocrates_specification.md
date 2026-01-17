# Hippocrates Language Model Context Protocol (Version 1.0)

## 1. Introduction

This document formalizes the **Hippocrates Language**, a domain-specific language (DSL) designed for defining medical care plans, protocols, and digital health interventions. The language emphasizes natural language readability, strict typing of physical units, and robust handling of temporal events.

## 2. Language Principles

* **Natural Language Syntax**: Statements mimic English sentences to ensure readability by medical professionals.
* **Type Safety & Units**: All numerical values representing physical quantities must have associated units.
* **Contextual Execution**: Scripts execute within a specific context (Patient, Timeframe).
* **Event-Driven**: The core runtime is an event loop reacting to time, value changes, and external triggers.
* **Completeness**: A plan describes a self-contained logic for a single subject (the Patient).

## 3. Formal Grammar (EBNF)

The following grammar defines the syntax of Hippocrates.

### 3.1. Basic Elements

```ebnf
(* Basic Tokens *)
digit = "0" | "1" | ... | "9";
integer = [ "-" ], digit, { digit };
float = integer, [ ".", digit, { digit } ];
word = character, { character };
string_literal = '"', { character }, '"';
identifier = word, { " ", word }; (* Identifiers can contain spaces *)

(* Basic Types *)
percentage = ( "0" | "100" | digit, [ digit ] ), "%";
time_unit = "year" | "month" | "week" | "day" | "hour" | "minute" | "second";
period_of_time = integer, " ", time_unit, [ "s" ]; (* e.g., 2 weeks *)

(* Units *)
temperature_unit = "°F" | "°C";
weight_unit = "mg" | "g" | "kg" | "lb";
volume_unit = "l" | "ml" | "fl oz";
unit = temperature_unit | weight_unit | volume_unit | time_unit;
```

### 3.2. Program Structure

```ebnf
hippocrates_file = 
    [ intended_use_chapter ],
    [ library_reference_chapter ],
    [ settings_chapter ],
    { definition_chapter },
    plan_chapter;

definition_chapter = 
    addressee_chapter |
    value_definition_chapter |
    period_definition_chapter |
    event_definition_chapter;
```

### 3.3. Values

Values are the core state containers.

```ebnf
value_definition = 
    identifier, " is ", value_type, " (", newline,
        { value_property },
    ")", ".";

value_type = "a number" | "an enumeration" | "a time indication";

value_property = 
    intended_use_prop |
    valid_units_prop |
    valid_values_prop |
    calculation_prop |
    reuse_prop;

(* Example: "body weight" is a number (...) *)
```

### 3.4. Logic and Control Flow

Statements inside a Plan or Paragraph.

```ebnf
statement = 
    action_statement |
    assignment_statement |
    conditional_statement |
    loop_statement;

(* Assignment *)
assignment_statement = identifier, " = ", expression, ".";

(* Actions *)
action_statement = 
    show_message_statement |
    ask_question_statement |
    start_period_statement;

(* Conditionals *)
conditional_statement = 
    "assess ", identifier, " >", newline,
        { assessment_case },
        [ "in all other cases >", block, "<" ],
    "<";

assessment_case = range_selector, " >", block, "<";

(* Ranges *)
range_selector = 
    "between (", expression, " ... ", expression, ")" |
    expression; (* Equality check *)
```

### 3.5. Events and Timing

```ebnf
event_listener = "catch ", event_trigger, " >", block, "<";

event_trigger = 
    "change of ", identifier |
    "every ", period_of_time |
    "at ", time_of_day;
```

## 4. Semantics and Type System

### 4.1. Unit Conversions

The runtime MUST ensure unit compatibility.

* **Automatic Conversion**: If a value is defined with `mg` and `g`, the engine must handle conversion automatically using standard physical constants.
* **Unit Groups**:
  * *Mass*: mg, g, kg, lb, oz
  * *Volume*: ml, l, fl oz, gal
  * *Time*: sec, min, hour, day, week, month, year
  * *Temperature*: C, F

### 4.2. Confidence and History

Every value in Hippocrates has meta-properties managed by the runtime:

* `value.timestamp`: When was this value last updated.
* `value.confidence`: A percentage (0-100%) indicating certainty.
  * Calculated values inherit the *lowest* confidence of their inputs.
  * Values decay in confidence over time if a `reuse` policy is defined.
  * **Rule**: Operations on values with `confidence < threshold` should trigger a `LowConfidenceError` or fallback to requesting a new value (Ask Question).

### 4.3. Context Resolution

Variables (Values) are resolved in the following order:

1. **Local Scope**: Variables defined within the current block (e.g., iterator variables).
2. **Global Scope**: Values defined in the `value definition` chapters.
3. **Library Scope**: Values imported from linked libraries.

## 5. Standard Library Proposal

Implicit definitions available in every plan.

### 5.1. System Values

* `now`: Current timestamp.
* `patient`: Reference to the primary subject.
* `plan_start_date`: Timestamp when the plan was activated.

### 5.2. Built-in Functions

* `count_of(value)`: Number of historical entries for a value.
* `average_of(value, period)`: Average of value over the last `period`.
* `trend_of(value)`: Enum (rising, falling, stable).

## 6. Execution Model

The Hippocrates Runtime functions as a **State Machine**.

1. **Load**: Parse script, build internal dependencies graph (DAG).
2. **Init**: Initialize all values to `unknown` or default; restore state from persistence.
3. **Loop**:
    * Check **Timers**: Are there any temporal events (`every day`, `at 10:00`)? -> Trigger Event.
    * Check **Inputs**: Did an external API update a value? -> Trigger `change of` Event.
    * evaluate **Rules**: If an Event triggered, execute the associated `block`.
    * **Side Effects**: Execute `show`, `ask`, or `send` commands via API callbacks.
