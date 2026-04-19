# SYRS-01: Hippocrates Language System Requirements

| Field              | Value                                                    |
|--------------------|----------------------------------------------------------|
| **Document ID**    | SYRS-01                                                  |
| **V-Model Level**  | System Requirements                                      |
| **Baseline**       | 1.0                                                      |
| **Status**         | Draft                                                    |
| **Source**         | `specification/hippocrates_specification.md` (Version 1.0)|
| **Date**           | 2026-03-20                                               |

## V-Model Preamble

This document is the **System Requirements** specification within the
Hippocrates V-Model development lifecycle. It sits between the Stakeholder
Requirements (STKR-*, defined in `00-stakeholder-requirements.md`) and the
lower-level Architecture / Detailed Design documents.

The content below adopts the Hippocrates Language Specification (Version 1.0)
verbatim. Every normative requirement (REQ-*) in the specification is a system
requirement. The traceability matrix in the next section maps each
specification section to the stakeholder requirements it satisfies.

Verification of these system requirements is performed at the **System
Integration Test** level (the mirror side of the V-Model), using the test
plans defined in the `test-plans/` directory.

---

## Traceability Matrix: Specification Sections to Stakeholder Requirements

| Spec Section                       | STKR Requirement(s)                                          |
|------------------------------------|---------------------------------------------------------------|
| §2 Language Principles             | SREQ-HIPP-002 (Readability), SREQ-HIPP-030 (No Comparison Operators), SREQ-HIPP-013 (Contextual Statements) |
| §3.1-3.3 Grammar                   | SREQ-HIPP-002 (Readability), SREQ-HIPP-011 (Readability Over Defaults)   |
| §3.2 Units                         | SREQ-HIPP-018 (Medical Domain), SREQ-HIPP-031 (Unit Discipline)          |
| §3.4 Values                        | SREQ-HIPP-012 (Meaningful Values), SREQ-HIPP-015 (Value Constraints)     |
| §3.5 Periods/Plans                 | SREQ-HIPP-005 (Event-Driven), SREQ-HIPP-020 (Plan Autonomy)              |
| §3.6 Statements                    | SREQ-HIPP-010 (Completeness), SREQ-HIPP-013 (Contextual Statements)      |
| §3.7 Actions/Questions             | SREQ-HIPP-001 (Care Plan Execution), SREQ-HIPP-005 (Event-Driven)        |
| §3.8 Events/Timing                 | SREQ-HIPP-005 (Event-Driven)                                       |
| §3.9 Communication                 | SREQ-HIPP-001 (Care Plan Execution), SREQ-HIPP-014 (Separation)          |
| §3.10 Medication                   | SREQ-HIPP-001 (Care Plan Execution), SREQ-HIPP-018 (Medical Domain)      |
| §3.11 Data Contexts                | SREQ-HIPP-013 (Contextual Statements)                              |
| §3.12 Expressions/Stats            | SREQ-HIPP-035 (Data Sufficiency)                                   |
| §4.1 Units/Conversion              | SREQ-HIPP-031 (Unit Discipline), SREQ-HIPP-018 (Medical Domain)          |
| §4.2 Required Properties           | SREQ-HIPP-015 (Value Constraints), SREQ-HIPP-011 (Readability Over Defaults) |
| §4.3 Data Flow                     | SREQ-HIPP-033 (Data Flow Validation)                               |
| §4.4 Coverage                      | SREQ-HIPP-010 (Completeness), SREQ-HIPP-032 (Exhaustive Coverage)        |
| §4.5 Range Compliance              | SREQ-HIPP-015 (Value Constraints)                                  |
| §4.6 Data Sufficiency              | SREQ-HIPP-035 (Data Sufficiency)                                   |
| §4.7 Date/Time                     | SREQ-HIPP-018 (Medical Domain)                                     |
| §5 Execution                       | SREQ-HIPP-001 (Care Plan Execution), SREQ-HIPP-005 (Event-Driven)        |
| §4.1 (REQ-HIPP-CORE-005)                 | SREQ-HIPP-037 (LLM-Correctable Error Diagnostics)                  |
| §5.1 Validation                    | SREQ-HIPP-004 (Validation Without Execution), SREQ-HIPP-037 (LLM-Correctable Error Diagnostics) |
| §5.2 Input Validation              | SREQ-HIPP-034 (Input Precision)                                    |
| §5.3 Meaning Evaluation            | SREQ-HIPP-012 (Meaningful Values)                                  |

---

## Adopted Specification

> The remainder of this document reproduces the Hippocrates Language
> Specification (Version 1.0) in its entirety. All REQ-* identifiers are
> normative system requirements.

---

# Hippocrates Language Model Context Protocol (Version 1.0)

## 1. Introduction

This document formalizes the **Hippocrates Language**, a domain-specific language (DSL) designed for defining medical care plans, protocols, and digital health interventions. The language emphasizes natural language readability, strict typing of physical units, and robust handling of temporal events.

## 2. Language Principles

* **Natural Language Syntax**: Statements mimic English sentences to ensure readability by medical professionals.
* **Type Safety & Units**: Informative — numeric literals are expressed as quantities with units (built-in or custom), e.g., `10 mg` or `7 days`. (See REQ-HIPP-UNITS-005.)
* **Contextual Execution**: Scripts execute within a specific context (Patient, Timeframe).
* **Event-Driven**: The core runtime is an event loop reacting to time, value changes, and external triggers.
* **Completeness**: A plan describes a self-contained logic for a single subject (the Patient).
* **Angle-Bracket Identifiers**: Informative — identifiers are written as `<...>`. (See REQ-HIPP-LANG-001.)
* **Indented Blocks**: Informative — any `:` that opens a block is followed by a newline and indented block. (See REQ-HIPP-LANG-004.)
* **Block Syntax**: Informative — a block is introduced with a trailing `:` and statements inside the block end with a `.`. (See REQ-HIPP-STMT-004, REQ-HIPP-STMT-005.)
* **No Comparison Operators**: Informative — use ranges (`min ... max`) instead of `<`, `>`, `<=`, `>=`. (See REQ-HIPP-LANG-003.)

Requirements:

#### REQ-HIPP-LANG-001 — Identifiers must use angle brackets

**Traces to:** SREQ-HIPP-002, SREQ-HIPP-030, SREQ-HIPP-013

identifiers must use angle brackets.

#### REQ-HIPP-LANG-002 — String literals must not contain angle brackets

**Traces to:** SREQ-HIPP-002, SREQ-HIPP-030, SREQ-HIPP-013

string literals must not contain angle brackets.

#### REQ-HIPP-LANG-003 — Comparison operators are not supported; use ranges

**Traces to:** SREQ-HIPP-002, SREQ-HIPP-030, SREQ-HIPP-013

comparison operators are not supported; use ranges.

#### REQ-HIPP-LANG-004 — Block openings require a newline and indented block

**Traces to:** SREQ-HIPP-002, SREQ-HIPP-030, SREQ-HIPP-013

block openings require a newline and indented block.

## 3. Formal Grammar (EBNF)

The following grammar defines the syntax of Hippocrates. Indentation is significant and is converted into explicit `INDENT`/`DEDENT` tokens by the parser.

### 3.1. Basic Elements

```ebnf
(* Layout *)
newline = "\n" | "\r\n";
comment = "(*", { character - "*)" }, "*)";

(* Basic Tokens *)
digit = "0" | "1" | ... | "9";
integer = [ "-" ], digit, { digit };
float = integer, ".", digit, { digit };
number = integer | float;

(* Strings and Identifiers *)
string_literal = '"', { character - ( '"' | "<" | ">" ) }, '"';
identifier = "<", { character - ">" }, ">";

(* Time *)
time_literal = digit, [ digit ], ":", digit, digit; (* H:MM or HH:MM *)
weekday = "Monday" | "Tuesday" | "Wednesday" | "Thursday" | "Friday" | "Saturday" | "Sunday";
time_indication = time_literal | weekday | "now";
date_literal = digit, digit, digit, digit, "-", digit, digit, "-", digit, digit;
datetime_literal = date_literal, " ", time_literal;
```

Informative: Identifiers are angle-bracketed. When a rule introduces an indented block (`:` followed by `indent`/`dedent`), a newline is used. Inline `:` forms appear only where explicitly shown (e.g., `documentation` strings). (See REQ-HIPP-LANG-001, REQ-HIPP-LANG-004, REQ-HIPP-BASIC-003.)
Informative: A block is introduced with a trailing `:` and statements inside the block end with a `.`. (See REQ-HIPP-STMT-004, REQ-HIPP-STMT-005.)

Requirements:

#### REQ-HIPP-BASIC-001 — Time indications parse for now, weekday, and time-of-day

**Traces to:** SREQ-HIPP-002, SREQ-HIPP-011

time indications parse for now, weekday, and time-of-day.

#### REQ-HIPP-BASIC-002 — Relative time expressions from now parse

**Traces to:** SREQ-HIPP-002, SREQ-HIPP-011

relative time expressions from now parse.

#### REQ-HIPP-BASIC-003 — Inline ':' forms are only allowed where explicitly shown

**Traces to:** SREQ-HIPP-002, SREQ-HIPP-011

inline ':' forms are only allowed where explicitly shown.

#### REQ-HIPP-BASIC-004 — Date/time literals parse for date and date-time forms

**Traces to:** SREQ-HIPP-002, SREQ-HIPP-011

date/time literals parse for date and date-time forms.

### 3.2. Units and Quantities

```ebnf
standard_unit =
    "°F" | "°C" | "%" |
    "mmHg" | "mg/dL" | "mmol/L" | "bpm" |
    "milligrams" | "milligram" | "mg" |
    "kilograms" | "kilogram" | "kg" |
    "grams" | "gram" | "g" |
    "pounds" | "pound" | "lb" |
    "ounces" | "ounce" | "oz" |
    "milliliters" | "milliliter" | "ml" |
    "liters" | "liter" | "l" |
    "fluid ounces" | "fluid ounce" | "fl oz" |
    "gallons" | "gallon" | "gal" |
    "centimeters" | "centimeter" | "cm" |
    "millimeters" | "millimeter" | "mm" |
    "kilometers" | "kilometer" | "km" |
    "meters" | "meter" | "m" |
    "inches" | "inch" |
    "feet" | "foot" |
    "miles" | "mile" |
    "years" | "months" | "weeks" | "days" | "hours" | "minutes" | "seconds" |
    "year" | "month" | "week" | "day" | "hour" | "minute" | "second";

custom_unit = identifier;
unit = standard_unit | custom_unit;

quantity = number, [ " " ], unit;
```

Informative: Numeric literals in user scripts are expressed as quantities with units; unitless numbers are invalid. (See REQ-HIPP-UNITS-005.)
Informative: Built-in units are reserved and cannot be redefined or aliased in unit definitions. (See REQ-HIPP-CORE-001.)

Informative: Precision is defined by the number of digits after the decimal point (e.g., `0` → precision 0, `0.0` → precision 1, `0.00` → precision 2). Each numeric valid value range uses the same precision for both bounds, and all ranges in a `valid values` block share the same precision. Integer ranges (e.g., `0 <points> ... 10 <points>`) use step size 1; decimal ranges (e.g., `0.0 mg ... 10.0 mg`) use the declared precision (step size `10^-precision`). (See REQ-HIPP-COVER-008, REQ-HIPP-COVER-012.)

Requirements:

#### REQ-HIPP-UNITS-001 — Custom unit pluralization canonicalizes values

**Traces to:** SREQ-HIPP-018, SREQ-HIPP-031

custom unit pluralization canonicalizes values.

#### REQ-HIPP-UNITS-002 — Standard units work in calculations

**Traces to:** SREQ-HIPP-018, SREQ-HIPP-031

standard units work in calculations.

#### REQ-HIPP-UNITS-003 — Custom unit abbreviations canonicalize values

**Traces to:** SREQ-HIPP-018, SREQ-HIPP-031

custom unit abbreviations canonicalize values.

#### REQ-HIPP-UNITS-004 — Custom unit quantities parse with definitions

**Traces to:** SREQ-HIPP-018, SREQ-HIPP-031

custom unit quantities parse with definitions.

#### REQ-HIPP-UNITS-005 — Numeric literals must include units

**Traces to:** SREQ-HIPP-018, SREQ-HIPP-031

numeric literals must include units.

### 3.3. Program Structure

```ebnf
hippocrates_file = { definition };

definition =
    unit_definition |
    drug_definition |
    addressee_definition |
    context_definition |
    plan_definition |
    period_definition |
    value_definition;
```

Requirements:

#### REQ-HIPP-PROG-001 — Multi-definition fixtures parse core definitions

**Traces to:** SREQ-HIPP-002, SREQ-HIPP-011

multi-definition fixtures parse core definitions.

### 3.4. Values

```ebnf
value_definition =
    identifier, " is ", value_type,
    [ ":", newline, indent, { value_property }, dedent ],
    [ "." ];

value_type =
    "a number" | "an enumeration" | "a string" | "a time indication" | "a date/time" |
    "a period" | "a plan" | "a drug" | "an addressee";

value_property =
    valid_values_prop |
    timeframe_prop |
    meaning_prop |
    question_prop |
    calculation_prop |
    reuse_prop |
    inheritance_prop |
    documentation_prop |
    unit_ref_prop |
    generic_property;

valid_values_prop =
    "valid values:", newline, indent, valid_values_block, dedent;

valid_values_block = { range_selector, [ ";" ], newline };

meaning_prop =
    "meaning of ", identifier, ":", newline, indent,
    valid_meanings_prop,
    { meaning_item_block },
    dedent;

meaning_item_block =
    meaning_assess_block |
    assessment_case;

valid_meanings_prop =
    "valid meanings:", newline, indent, valid_meanings_line, { valid_meanings_line }, dedent;

valid_meanings_line = meaning_item, { ";", meaning_item }, [ ";" ], ".";
meaning_item = identifier;

meaning_assess_block =
    "assess meaning of ", identifier, ":", newline, indent, { assessment_case }, dedent;

question_prop = "question:", newline, indent, { statement }, dedent;

calculation_prop = "calculation:", newline, indent, { statement }, dedent;

reuse_prop = "reuse:", newline, indent, reuse_stmt, dedent;
reuse_stmt = "reuse period of value is ", quantity, [ "." ];

inheritance_prop =
    "definition is the same as for ", identifier,
    [ " except:", newline, indent, { value_property }, dedent ];

documentation_prop = "documentation:", newline, indent, "english", flexible_string_block, dedent;
flexible_string_block =
    ":", newline, indent, string_literal, [ "." ], dedent |
    ":", string_literal, [ "." ];

timeframe_prop = "timeframe:", newline, indent, timeframe_line, { timeframe_line }, dedent;
timeframe_line = timeframe_selector, { ";", timeframe_selector }, newline;

unit_ref_prop = "unit", (" is " | ":"), unit;

generic_property = identifier, flexible_property_content;
flexible_property_content =
    ":", newline, indent, property_content, dedent |
    ":", property_line;
property_content = { character };
property_line = { character - newline };
```

Informative: `meaning of <value>:` blocks must include the target identifier; the shorthand `meaning:` is not part of the language. (See REQ-HIPP-VALUE-010.)
Informative: `meaning of <value>:` blocks must declare `valid meanings:` before meaning assessments. (See REQ-HIPP-VALUE-011.)
Informative: Meaning labels and `valid meanings` entries are identifiers (angle brackets), not string literals. (See REQ-HIPP-VALUE-012.)
Informative: `meaning of <value>:` blocks and `assess meaning of <value>` blocks are value properties only; they are not valid statements inside plans or other block contexts. (See REQ-HIPP-VALUE-009.)

Requirements:

#### REQ-HIPP-VALUE-001 — Value definitions parse from fixtures

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

value definitions parse from fixtures.

#### REQ-HIPP-VALUE-002 — Value type variants parse correctly

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

value type variants parse correctly.

#### REQ-HIPP-VALUE-003 — Unit properties parse in numeric values

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

unit properties parse in numeric values.

#### REQ-HIPP-VALUE-004 — Value timeframe properties parse

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

value timeframe properties parse.

#### REQ-HIPP-VALUE-005 — Inheritance properties parse with overrides

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

inheritance properties parse with overrides.

#### REQ-HIPP-VALUE-006 — Documentation properties parse in inline and block forms

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

documentation properties parse in inline and block forms.

#### REQ-HIPP-VALUE-007 — Custom properties parse as generic properties

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

custom properties parse as generic properties.

#### REQ-HIPP-VALUE-008 — Date/time value types parse correctly

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

date/time value types parse correctly.

#### REQ-HIPP-VALUE-009 — Meaning assessments are only allowed in value definition blocks

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

meaning assessments are only allowed in value definition blocks.

#### REQ-HIPP-VALUE-010 — Meaning properties require an explicit target identifier

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

meaning properties require an explicit target identifier.

#### REQ-HIPP-VALUE-011 — Meaning properties must declare valid meanings

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

meaning properties must declare valid meanings.

#### REQ-HIPP-VALUE-012 — Meaning labels must be identifiers (angle brackets)

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

meaning labels must be identifiers (angle brackets).

#### REQ-HIPP-VALUE-013 — Enumeration valid values are identifiers (angle brackets)

**Traces to:** SREQ-HIPP-012, SREQ-HIPP-015

enumeration valid values are identifiers (angle brackets).

### 3.5. Periods and Plans

```ebnf
period_definition =
    identifier, " is a period:", newline, indent, { period_property }, dedent;

period_property =
    "timeframe:", newline, indent, timeframe_line, { timeframe_line }, dedent |
    "customization:", newline, indent, block_text, dedent;

plan_definition =
    identifier, " is a plan:", newline, indent, { plan_block }, dedent;

plan_block =
    before_plan_block |
    after_plan_block |
    trigger_block |
    event_block;

before_plan_block = "before plan:", newline, indent, { statement }, dedent;

after_plan_block = "after plan:", newline, indent, { statement }, dedent;

trigger_block = event_trigger, ":", newline, indent, { statement }, dedent;

event_block = identifier, " with ", event_trigger, ":", newline, indent, { statement }, dedent;
```

Requirements:

#### REQ-HIPP-PLAN-001 — Period definitions parse by name

**Traces to:** SREQ-HIPP-005, SREQ-HIPP-020

period definitions parse by name.

#### REQ-HIPP-PLAN-002 — Period timeframe lines parse with range selectors

**Traces to:** SREQ-HIPP-005, SREQ-HIPP-020

period timeframe lines parse with range selectors.

### 3.6. Statements, Assessments, and Ranges

```ebnf
statement =
    timeframe_block |
    documentation_prop |
    context_block |
    conditional |
    assignment |
    meaning_assignment |
    constraint |
    action |
    newline;

assignment = identifier, " = ", expression, ".";
meaning_assignment = "meaning of value = ", identifier, ".";

conditional =
    "assess ", ( confidence_target | expression ), ":", newline, indent,
    { assessment_case },
    dedent;

confidence_target = "confidence of ", identifier;

assessment_case = selector_list, ":", newline, indent, block, dedent;
selector_list = range_selector, { ";", range_selector };

range_selector =
    "Not enough data" |
    "between ", expression, " ... ", expression |
    expression, " ... ", expression |
    expression;

timeframe_selector =
    "between ", expression, " ... ", expression |
    expression, " ... ", expression |
    identifier;

Informative: Timeframe selectors include a start and an end; single time indications (e.g., `now`) are not valid timeframes. When a timeframe selector uses an identifier, it refers to a period definition that provides the underlying start/end bounds. (See REQ-HIPP-STMT-002, REQ-HIPP-STMT-003.)

constraint = expression, constraint_operator, range_selector, ".";
constraint_operator = "is" | "during" | "after";

block = { statement };

context_block = "context", [ " for analysis" ], ":", newline, indent, { context_item | statement }, dedent;

Informative: Statements inside indented blocks terminate with a period (`.`). Blocks are introduced with a colon (`:`). (See REQ-HIPP-STMT-004, REQ-HIPP-STMT-005.)

timeframe_block =
    "timeframe", [ " for analysis" ],
    constraint_operator, timeframe_selector, { constraint_operator, timeframe_selector },
    ":", newline, indent, { statement }, dedent;
```

Requirements:

#### REQ-HIPP-STMT-001 — Timeframe blocks parse with nested statements

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-013

timeframe blocks parse with nested statements.

#### REQ-HIPP-STMT-002 — Timeframe selectors require a start and end

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-013

timeframe selectors require a start and end.

#### REQ-HIPP-STMT-003 — Timeframe selector identifiers must refer to defined periods

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-013

timeframe selector identifiers must refer to defined periods.

#### REQ-HIPP-STMT-004 — Statements inside blocks must terminate with a period

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-013

statements inside blocks must terminate with a period.

#### REQ-HIPP-STMT-005 — Blocks must be introduced with a colon

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-013

blocks must be introduced with a colon.

### 3.7. Actions and Questions

```ebnf
action =
    message_action |
    ask_question |
    listen_for |
    send_info |
    question_modifier |
    start_period |
    simple_command;

message_action =
    information_message |
    warning_message |
    urgent_warning_message;

information_message = information_message_block | information_message_inline;
warning_message = warning_message_block | warning_message_inline;
urgent_warning_message = urgent_warning_message_block | urgent_warning_message_inline;

information_message_block =
    "information", [ " to ", addressee_list ],
    ( message_block | ( message_content_line, message_property_block ) );

information_message_inline =
    "information", [ " to ", addressee_list ],
    message_content_line, ".";

warning_message_block =
    "warning", [ " to ", addressee_list ],
    ( message_block | ( message_content_line, message_property_block ) );

warning_message_inline =
    "warning", [ " to ", addressee_list ],
    message_content_line, ".";

urgent_warning_message_block =
    "urgent warning", [ " to ", addressee_list ],
    ( message_block | ( message_content_line, message_property_block ) );

urgent_warning_message_inline =
    "urgent warning", [ " to ", addressee_list ],
    message_content_line, ".";

addressee_list = identifier, { ";", identifier };

message_content_line = expression;
message_property = message_expiration;
message_block =
    ":", newline, indent, message_block_line, { message_block_line }, dedent;
message_block_line = message_content_line | message_property;
message_property_block =
    ":", newline, indent, message_property, { message_property }, dedent;

ask_question = ask_question_block | ask_question_inline;

ask_question_block =
    "ask", [ " for" | " patient" | " physician" ], ( string_literal | identifier ),
    flexible_block;

ask_question_inline =
    "ask", [ " for" | " patient" | " physician" ], ( string_literal | identifier ), ".";

flexible_block = ":", newline, indent, { statement }, dedent;

listen_for = "listen for ", identifier, ":", newline, indent, { statement }, dedent;

send_info = "send information ", string_literal, { expression | newline }, ".";

start_period = "start ", identifier, ".";

simple_command = identifier, ".";

message_expiration = "message expires after ", message_expiration_value, ".";
message_expiration_value = quantity | identifier;

question_modifier =
    "question expires after ", period_expr, flexible_block |
    "question expires after ", period_expr, "." |
    validate_modifier |
    "type of question is ", string_literal, "." |
    "style of question is ", identifier, "." |
    "question style is visual analogue scale:", newline, indent, vas_block, dedent;

validate_modifier =
    "validate answer ", validation_mode, [ " within ", quantity ], ( flexible_block | "." );
validation_mode = "once" | "twice";

vas_block = { best_value_def | best_label_def | worst_value_def | worst_label_def };

best_value_def = "best value is ", quantity, [ "." ], newline;
best_label_def = "text for best value is ", string_literal, [ "." ], newline;
worst_value_def = "worst value is ", quantity, [ "." ], newline;
worst_label_def = "text for worst value is ", string_literal, [ "." ], newline;
```

Question waits do not block subsequent loop triggers. If the next scheduled trigger occurs before an answer arrives, the engine resumes the loop with the question still pending (the block may re-ask or continue). If a question expires without an answer, an optional `question expires after` block runs at expiration time and can send a reminder or log a message.

Requirements:

#### REQ-HIPP-ACT-001 — Question configuration parses and validates references

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

question configuration parses and validates references.

#### REQ-HIPP-ACT-002 — Message expiration attaches to information, warning, and urgent warning actions

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

message expiration attaches to information, warning, and urgent warning actions.

#### REQ-HIPP-ACT-003 — Question modifiers parse (validate/type/style/expire)

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

question modifiers parse (validate/type/style/expire).

#### REQ-HIPP-ACT-004 — Validate answer within parsing attaches to ask blocks

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

validate answer within parsing attaches to ask blocks.

#### REQ-HIPP-ACT-005 — Listen/send/start/simple command actions parse

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

listen/send/start/simple command actions parse.

#### REQ-HIPP-ACT-006 — Question expiration blocks parse with reminder statements

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

question expiration blocks parse with reminder statements.

#### REQ-HIPP-ACT-007 — Question expiration supports until event triggers

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

question expiration supports until event triggers.

#### REQ-HIPP-ACT-008 — Information, warning, and urgent warning are accepted as message action keywords

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

information, warning, and urgent warning are accepted as message action keywords.

#### REQ-HIPP-ACT-009 — Message actions accept semicolon-separated addressee lists

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

message actions accept semicolon-separated addressee lists.

#### REQ-HIPP-ACT-010 — `after plan:` block syntax and semantics. The `after plan:` block is a plan block that executes its statements exactly once when the event loop terminates (all triggers exhausted or simulation time limit reached)

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

`after plan:` block syntax and semantics. The `after plan:` block is a plan block that executes its statements exactly once when the event loop terminates (all triggers exhausted or simulation time limit reached).

### 3.8. Events and Timing

```ebnf
ordinal = "other" | "second" | "third" | "fourth" | "fifth" | "sixth" | "seventh" | "eighth" | "ninth" | "tenth";

bare_unit = "day" | "week" | "month" | "year" | "hour" | "minute" | "second";

event_trigger =
    "change of ", identifier |
    "begin of ", identifier |
    "every ", ( ordinal, bare_unit | bare_unit | quantity, [ identifier ] ), [ " at ", time_literal ], [ " for ", quantity ] |
    "every ", ( identifier | weekday ), [ " at ", time_literal ], [ " after ", identifier ], [ " for ", quantity ];

period_expr = quantity | "until ", event_trigger;
```

Requirements:

#### REQ-HIPP-EVT-001 — Event triggers parse for change/start/periodic

**Traces to:** SREQ-HIPP-005

event triggers parse for change/start/periodic.

#### REQ-HIPP-EVT-002 — Event blocks attach statements to triggers

**Traces to:** SREQ-HIPP-005

event blocks attach statements to triggers.

#### REQ-HIPP-EVT-003 — Scheduler computes next occurrence for periods

**Traces to:** SREQ-HIPP-005

scheduler computes next occurrence for periods.

#### REQ-HIPP-EVT-004 — Periodic triggers parse duration and offsets

**Traces to:** SREQ-HIPP-005

periodic triggers parse duration and offsets.

#### REQ-HIPP-EVT-005 — Periodic triggers accept an optional `at <time_literal>` clause to pin execution to a specific time of day

**Traces to:** SREQ-HIPP-005

periodic triggers accept an optional `at <time_literal>` clause to pin execution to a specific time of day.

#### REQ-HIPP-EVT-006 — Periodic triggers using a named period identifier with a `for` duration fire at every occurrence of that period within the duration window

**Traces to:** SREQ-HIPP-005

periodic triggers using a named period identifier with a `for` duration fire at every occurrence of that period within the duration window.

#### REQ-HIPP-EVT-007 — The grammar shall accept bare unit names (`every day`, `every week`) as shorthand for `every 1 day`, `every 1 week`. The grammar shall accept ordinals (`every second day`, `every third week`, `every other day`) as shorthand for the corresponding numeric interval. Supported ordinals: other/second (2), third (3), fourth (4), fifth (5), sixth (6), seventh (7), eighth (8), ninth (9), tenth (10)

**Traces to:** SREQ-HIPP-005

the grammar shall accept bare unit names (`every day`, `every week`) as shorthand for `every 1 day`, `every 1 week`. The grammar shall accept ordinals (`every second day`, `every third week`, `every other day`) as shorthand for the corresponding numeric interval. Supported ordinals: other/second (2), third (3), fourth (4), fifth (5), sixth (6), seventh (7), eighth (8), ninth (9), tenth (10).

### 3.9. Communication & Actors

```ebnf
addressee_definition =
    identifier, (" is an addressee" | " is an addressee group"), ":", newline, indent,
    { contact_info_prop | grouped_addressees_prop | contact_logic_prop | after_consent_prop },
    dedent;

contact_info_prop = "contact information:", newline, indent, { contact_detail }, dedent;
contact_detail = contact_type, " is ", string_literal;
contact_type = "email" | "phone" | "hippocrates id";

after_consent_prop = "after consent has been rejected:", newline, indent, block, dedent;

grouped_addressees_prop = "grouped addressees are ", identifier, { ";", identifier };

contact_logic_prop = "order of contacting:", newline, indent, contact_logic, dedent;
contact_logic =
    "contact all addressees in parallel" |
    "sequence of contacting is ", identifier, { ";", identifier };
```

Requirements:

#### REQ-HIPP-COMM-001 — Addressee groups and contact logic parse

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-014

addressee groups and contact logic parse.

#### REQ-HIPP-COMM-002 — Contact details and sequence ordering parse

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-014

contact details and sequence ordering parse.

### 3.10. Medication

```ebnf
drug_definition =
    identifier, " is a drug:", newline, indent,
    { ingredients_block | dosage_block | admin_block | interaction_block },
    dedent;

ingredients_block = "ingredients:", newline, indent, { ingredient }, dedent;
ingredient = identifier, quantity;

dosage_block = "dosage safety:", newline, indent, { dosage_rule }, dedent;
dosage_rule =
    "maximum single dose = ", expression |
    "maximum daily dose = ", expression |
    "minimum time between doses = ", expression;

admin_block = "administration:", newline, indent, { admin_rule }, dedent;
admin_rule =
    "form of administration is ", ( identifier | string_literal ) |
    identifier, quantity, " after ", identifier;

interaction_block = "interactions:", newline, indent, { interaction_rule }, dedent;
interaction_rule = "assess interaction with ", identifier, ":", newline, indent, block, dedent;
```

Requirements:

#### REQ-HIPP-MED-001 — Drug definition validation rejects undefined units

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-018

drug definition validation rejects undefined units.

#### REQ-HIPP-MED-002 — Drug interaction properties parse

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-018

drug interaction properties parse.

#### REQ-HIPP-MED-003 — Dosage safety and administration rules parse

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-018

dosage safety and administration rules parse.

### 3.11. Data Contexts

```ebnf
context_definition = "context:", newline, indent, { context_item }, dedent;

context_item =
    "timeframe:", timeframe_selector |
    "data:", identifier |
    "value filter:", assessment_case;
```

Requirements:

#### REQ-HIPP-CTX-001 — Context definitions parse timeframe/data/value filter items

**Traces to:** SREQ-HIPP-013

context definitions parse timeframe/data/value filter items.

#### REQ-HIPP-CTX-002 — Context blocks parse data/value filters and nested statements

**Traces to:** SREQ-HIPP-013

context blocks parse data/value filters and nested statements.

#### REQ-HIPP-CTX-003 — Context for analysis executes with scoped timeframe

**Traces to:** SREQ-HIPP-013

context for analysis executes with scoped timeframe.

### 3.12. Expressions and Statistical Analysis

```ebnf
expression = term, { infix_op, term };

term =
    date_diff |
    "meaning of ", identifier |
    quantity, relative_time_modifier |
    statistical_func |
    quantity |
    datetime_literal |
    date_literal |
    time_indication |
    string_literal |
    identifier |
    "(", expression, ")";

statistical_func =
    ("count of" | "min of" | "max of" | "trend of"), identifier, [ " is ", term ] |
    "average of", identifier, (" over " | " for "), quantity;

infix_op = "+" | "-" | "*" | "/";
relative_time_modifier = "ago" | "from now";
date_diff_unit = "year" | "month" | "day" | "hour" | "minute" | "years" | "months" | "days" | "hours" | "minutes";
date_diff = date_diff_unit, " between ", expression, " and ", expression;
```

Informative: Statistical functions are evaluated within an analysis timeframe. Use a `timeframe for analysis` block or a `context for analysis` block that provides a `timeframe:` item. (See REQ-HIPP-EXPR-005.)

Requirements:

#### REQ-HIPP-EXPR-001 — Statistical function expressions parse in assignments

**Traces to:** SREQ-HIPP-035

statistical function expressions parse in assignments.

#### REQ-HIPP-EXPR-002 — Timeframe filtering applies to statistical evaluations

**Traces to:** SREQ-HIPP-035

timeframe filtering applies to statistical evaluations.

#### REQ-HIPP-EXPR-003 — Timeframe variants resolve counts over different windows

**Traces to:** SREQ-HIPP-035

timeframe variants resolve counts over different windows.

#### REQ-HIPP-EXPR-004 — Trend analysis evaluates statistical trends over timeframes

**Traces to:** SREQ-HIPP-035

trend analysis evaluates statistical trends over timeframes.

#### REQ-HIPP-EXPR-005 — Statistical functions require an analysis timeframe context

**Traces to:** SREQ-HIPP-035

statistical functions require an analysis timeframe context.

#### REQ-HIPP-EXPR-006 — Date diff expressions parse

**Traces to:** SREQ-HIPP-035

date diff expressions parse.

#### REQ-HIPP-EXPR-007 — Meaning-of expressions parse

**Traces to:** SREQ-HIPP-035

meaning-of expressions parse.

## 4. Semantics and Type System

### 4.1. Core Unit Groups and Conversion

The runtime recognizes the standard units listed in the grammar. Automatic conversions are supported within the following groups:

* **Mass**: mg, g, kg, lb, oz
* **Length**: m, cm, mm, km, inch, foot, mile
* **Volume**: ml, l, fl oz, gal
* **Time**: second, minute, hour, day, week, month, year (including plural forms)
* **Temperature**: °C, °F

Additional standard units are recognized as distinct groups:

* **Percent**: %
* **Pressure**: mmHg
* **Clinical**: bpm, mg/dL, mmol/L (mg/dL and mmol/L are convertible)

Informative: The runtime checks that values compared have compatible units (belong to the same group). Calculations and assignments require exact unit matches and the same numeric precision as the target value. (See REQ-HIPP-CORE-002, REQ-HIPP-CORE-003.)

#### Unit Normalization

Informative: For custom units (e.g., `points`, `tablets`), pluralization and abbreviations are defined explicitly. Without a definition, `10 <point>` and `10 <points>` are treated as different units. (See REQ-HIPP-UNITS-001, REQ-HIPP-UNITS-003.)

```hippocrates
<point> is a unit:
    plural is <points>
    abbreviation is "pts"
```

Requirements:

#### REQ-HIPP-CORE-001 — Built-in units cannot be redefined

**Traces to:** SREQ-HIPP-031, SREQ-HIPP-018

built-in units cannot be redefined.

#### REQ-HIPP-CORE-002 — Unit conversions are supported within compatible groups

**Traces to:** SREQ-HIPP-031, SREQ-HIPP-018

unit conversions are supported within compatible groups.

#### REQ-HIPP-CORE-003 — Calculations and assignments require matching units and precision

**Traces to:** SREQ-HIPP-031, SREQ-HIPP-018

calculations and assignments require matching units and precision.

#### REQ-HIPP-CORE-005 — Parse errors shall include a human-readable description of the expected syntax at the error location, mapping internal grammar rule names to user-facing terms (e.g., `plan_definition` → `plan declaration with colon`). The raw PEG rule names shall not appear in error messages

**Traces to:** SREQ-HIPP-037

parse errors shall include a human-readable description of the expected syntax at the error location, mapping internal grammar rule names to user-facing terms (e.g., `plan_definition` → `plan declaration with colon`). The raw PEG rule names shall not appear in error messages.

### 4.2. Required Properties

* **Numbers and Enumerations**: Informative — `valid values` are defined. (See REQ-HIPP-REQP-006.)
* **Numbers**: Informative — a unit is defined (via `unit is ...` or by using quantities in `valid values`). (See REQ-HIPP-REQP-001, REQ-HIPP-REQP-003.)
* **Asking**: Informative — `ask` is only valid when a value has a `question` property. (See REQ-HIPP-REQP-004.)
* **Valid Values**: Informative — valid value ranges are non-overlapping (including numeric, date/time, and time-of-day ranges); use disjoint ranges when multiple intervals are needed. (See REQ-HIPP-REQP-007.)
* **Enumerations**: Informative — enumeration valid values are identifiers (angle brackets), not string literals. (See REQ-HIPP-VALUE-013.)

Requirements:

#### REQ-HIPP-REQP-001 — Numeric valid values require units

**Traces to:** SREQ-HIPP-015, SREQ-HIPP-011

numeric valid values require units.

#### REQ-HIPP-REQP-002 — Assessment ranges require units

**Traces to:** SREQ-HIPP-015, SREQ-HIPP-011

assessment ranges require units.

#### REQ-HIPP-REQP-003 — Numeric definitions require units

**Traces to:** SREQ-HIPP-015, SREQ-HIPP-011

numeric definitions require units.

#### REQ-HIPP-REQP-004 — Ask requires a question property on the value

**Traces to:** SREQ-HIPP-015, SREQ-HIPP-011

ask requires a question property on the value.

#### REQ-HIPP-REQP-005 — Unit requirement errors report line numbers

**Traces to:** SREQ-HIPP-015, SREQ-HIPP-011

unit requirement errors report line numbers.

#### REQ-HIPP-REQP-006 — Numbers and enumerations must define valid values

**Traces to:** SREQ-HIPP-015, SREQ-HIPP-011

numbers and enumerations must define valid values.

#### REQ-HIPP-REQP-007 — Valid value ranges must not overlap

**Traces to:** SREQ-HIPP-015, SREQ-HIPP-011

valid value ranges must not overlap.

### 4.3. Data Flow and Validity

* Informative — a value is not used before it has valid content. (See REQ-HIPP-FLOW-001.)
* Informative — values gain valid content by being assigned, asked, or provided by `listen for` or `context data:`. (See REQ-HIPP-FLOW-004.)
* Informative — calculation properties describe how a value is derived but do not implicitly seed it; plans assign or ask before use. (See REQ-HIPP-FLOW-002.)
* Informative — statistical functions read history and do not require local initialization of the referenced value. (See REQ-HIPP-FLOW-003.)
* Informative — `meaning of <value>` is valid only if the value is already initialized or has a `question` property so the runtime can ask when missing. (See REQ-HIPP-FLOW-005.)

Requirements:

#### REQ-HIPP-FLOW-001 — Values cannot be used before assignment

**Traces to:** SREQ-HIPP-033

values cannot be used before assignment.

#### REQ-HIPP-FLOW-002 — Calculation properties do not seed values

**Traces to:** SREQ-HIPP-033

calculation properties do not seed values.

#### REQ-HIPP-FLOW-003 — Statistical functions do not require local initialization

**Traces to:** SREQ-HIPP-033

statistical functions do not require local initialization.

#### REQ-HIPP-FLOW-004 — Listen for and context data initialize values

**Traces to:** SREQ-HIPP-033

listen for and context data initialize values.

#### REQ-HIPP-FLOW-005 — Meaning-of expressions require an askable value when not initialized

**Traces to:** SREQ-HIPP-033

meaning-of expressions require an askable value when not initialized.

### 4.4. Assessment Coverage

Meaning assessments are declared as `meaning of <value>:` and must include `valid meanings:`; they may include `assess meaning of <value>` blocks. Meaning labels are identifiers (angle brackets). Within a meaning case, a bare identifier statement (e.g., `<light>.`) assigns that meaning label, equivalent to `meaning of value = <light>.`.

* Informative — `assess` blocks, `meaning` cases, and assessments over statistical results fully cover the valid range of the target/output. (See REQ-HIPP-COVER-001, REQ-HIPP-COVER-002, REQ-HIPP-COVER-003.)
* Informative — for enumerations, all valid values are covered. (See REQ-HIPP-COVER-005.)
* Informative — for `trend of <value>`, all cases (`"increase"`, `"decrease"`, `"stable"`) are covered. (See REQ-HIPP-COVER-011.)
* Informative — when `valid meanings` are declared, meaning assessments use all declared labels and no others. (See REQ-HIPP-COVER-013, REQ-HIPP-COVER-014.)

Requirements:

#### REQ-HIPP-COVER-001 — Meaning ranges must cover valid values (integer gaps)

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

meaning ranges must cover valid values (integer gaps).

#### REQ-HIPP-COVER-002 — Meaning ranges must cover valid values (float gaps)

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

meaning ranges must cover valid values (float gaps).

#### REQ-HIPP-COVER-003 — Disjoint valid ranges are allowed when fully covered

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

disjoint valid ranges are allowed when fully covered.

#### REQ-HIPP-COVER-004 — Overlapping numeric assessment ranges are invalid

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

overlapping numeric assessment ranges are invalid.

#### REQ-HIPP-COVER-005 — Duplicate enumeration cases are invalid

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

duplicate enumeration cases are invalid.

#### REQ-HIPP-COVER-006 — Gap detection reports missing integer spans

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

gap detection reports missing integer spans.

#### REQ-HIPP-COVER-007 — Gap detection reports missing float spans

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

gap detection reports missing float spans.

#### REQ-HIPP-COVER-008 — Coverage gaps respect precision for float and integer ranges

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

coverage gaps respect precision for float and integer ranges.

#### REQ-HIPP-COVER-009 — Overlapping ranges are rejected

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

overlapping ranges are rejected.

#### REQ-HIPP-COVER-010 — Missing coverage yields a validation error

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

missing coverage yields a validation error.

#### REQ-HIPP-COVER-011 — Trend assessments require full coverage

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

trend assessments require full coverage.

#### REQ-HIPP-COVER-012 — Numeric valid value ranges use consistent precision across bounds and intervals

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

numeric valid value ranges use consistent precision across bounds and intervals.

#### REQ-HIPP-COVER-013 — Valid meanings must be fully used across meaning assessments

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

valid meanings must be fully used across meaning assessments.

#### REQ-HIPP-COVER-014 — Meaning labels must be drawn from declared valid meanings

**Traces to:** SREQ-HIPP-010, SREQ-HIPP-032

meaning labels must be drawn from declared valid meanings.

### 4.5. Range Compliance (Pre-Run Validation)

Informative: Before execution, the runtime validates that calculated and assigned values remain within their declared ranges. If the computed range can exceed the valid values, validation fails. (See REQ-HIPP-RANGE-001, REQ-HIPP-RANGE-002.)

Requirements:

#### REQ-HIPP-RANGE-001 — Interval math supports range compliance checks

**Traces to:** SREQ-HIPP-015

interval math supports range compliance checks.

#### REQ-HIPP-RANGE-002 — Assignment range compliance fails when out of bounds

**Traces to:** SREQ-HIPP-015

assignment range compliance fails when out of bounds.

### 4.6. Data Sufficiency

Informative: Calculations involving history use `Not enough data` when the available history is shorter than the requested timeframe. This is handled explicitly in assessments. (See REQ-HIPP-SUFF-001, REQ-HIPP-SUFF-002, REQ-HIPP-SUFF-003.)
Informative: `Not enough data` assessments are only used when the assessed target is derived from statistical functions. (See REQ-HIPP-SUFF-004.)

Requirements:

#### REQ-HIPP-SUFF-001 — Timeframe calculations require Not enough data handling

**Traces to:** SREQ-HIPP-035

timeframe calculations require Not enough data handling.

#### REQ-HIPP-SUFF-002 — Not enough data handling satisfies sufficiency

**Traces to:** SREQ-HIPP-035

Not enough data handling satisfies sufficiency.

#### REQ-HIPP-SUFF-003 — Runtime evaluation returns NotEnoughData when history is insufficient

**Traces to:** SREQ-HIPP-035

runtime evaluation returns NotEnoughData when history is insufficient.

#### REQ-HIPP-SUFF-004 — Not enough data is only allowed for statistical assessments

**Traces to:** SREQ-HIPP-035

Not enough data is only allowed for statistical assessments.

### 4.7 Date/Time Semantics

* Informative — date/time ranges compare full date-time values; time-only ranges compare time-of-day and may wrap over midnight. (See REQ-HIPP-DTIME-001.)
* Informative — date diffs return whole calendar months/years and fractional day/hour/minute quantities based on elapsed time. (See REQ-HIPP-DTIME-002.)

Requirements:

#### REQ-HIPP-DTIME-001 — Date/time valid value ranges evaluate using date/time and time-of-day semantics

**Traces to:** SREQ-HIPP-018

date/time valid value ranges evaluate using date/time and time-of-day semantics.

#### REQ-HIPP-DTIME-002 — Date diff expressions evaluate to quantities in requested units

**Traces to:** SREQ-HIPP-018

date diff expressions evaluate to quantities in requested units.

## 5. Execution Model

The Hippocrates Runtime functions as a **State Machine**.

1. **Load**: Parse script, build internal dependencies graph (DAG).
2. **Init**: Initialize all values to `unknown` or default; restore state from persistence.
3. **Loop**:
    * Check **Timers**: Are there any temporal events (`every 1 day`, `every Monday`)? -> Trigger Event.
    * Check **Inputs**: Did an external API update a value? -> Trigger `change of` Event.
    * Evaluate **Rules**: If an Event triggered, execute the associated `block`.
    * **Side Effects**: Execute `information`, `warning`, `urgent warning`, `ask`, or `send information` commands via API callbacks.
    * Informative — message delivery uses the message callback; if it is not provided, the runtime logs a warning. (See REQ-HIPP-EXEC-003.)

Requirements:

#### REQ-HIPP-EXEC-001 — Runtime executes assignments and actions in order

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

runtime executes assignments and actions in order.

#### REQ-HIPP-EXEC-002 — Reuse timeframes prevent re-asking within the validity window

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

reuse timeframes prevent re-asking within the validity window.

#### REQ-HIPP-EXEC-003 — Runtime emits a warning when a message action executes without a message callback

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

runtime emits a warning when a message action executes without a message callback.

#### REQ-HIPP-EXEC-004 — Simulation mode executes plans at accelerated speed without real-time delays

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

simulation mode executes plans at accelerated speed without real-time delays.

#### REQ-HIPP-EXEC-005 — When a periodic trigger includes an `at` time, the first execution is scheduled at that time on the first eligible day, and subsequent executions recur at that same time each interval

**Traces to:** SREQ-HIPP-001, SREQ-HIPP-005

when a periodic trigger includes an `at` time, the first execution is scheduled at that time on the first eligible day, and subsequent executions recur at that same time each interval.

### 5.1 Validation Logic

Before execution, the runtime validates that:

1. All `assess` blocks and `meaning` cases cover the complete valid range of the target value.
2. No values are used before they are initialized or asked.
3. All referenced variables and units are compatible.

Requirements:

#### REQ-HIPP-VALID-001 — Full-plan validation passes for a complete plan

**Traces to:** SREQ-HIPP-004, SREQ-HIPP-037

full-plan validation passes for a complete plan.

#### REQ-HIPP-VALID-002 — The validator shall detect references to undeclared variables, addressees, units, and periods in plan blocks. Each undefined reference shall produce an error naming the undefined identifier and listing available definitions of that type

**Traces to:** SREQ-HIPP-037

the validator shall detect references to undeclared variables, addressees, units, and periods in plan blocks. Each undefined reference shall produce an error naming the undefined identifier and listing available definitions of that type.

#### REQ-HIPP-VALID-003 — Validation errors shall include a `suggestion` field with an actionable fix description. Coverage gaps shall suggest the exact missing range. Overlap errors shall suggest which range to adjust. Data flow errors shall suggest adding an `ask for` statement

**Traces to:** SREQ-HIPP-037

validation errors shall include a `suggestion` field with an actionable fix description. Coverage gaps shall suggest the exact missing range. Overlap errors shall suggest which range to adjust. Data flow errors shall suggest adding an `ask for` statement.

### 5.2 Input Validation

Informative: Integer valid values (e.g., `0 hours ... 24 hours`) reject fractional inputs; decimal ranges allow their declared precision. (See REQ-HIPP-INPUT-001.)

Requirements:

#### REQ-HIPP-INPUT-001 — Numeric answers must respect the decimal precision implied by valid values

**Traces to:** SREQ-HIPP-034

numeric answers must respect the decimal precision implied by valid values.

### 5.3 Meaning Evaluation

Informative: Meaning evaluation selects the first matching meaning case for the value and returns the assigned meaning label. Nested `assess` blocks are evaluated to resolve the final meaning. (See REQ-HIPP-MEAN-001, REQ-HIPP-MEAN-003.)
Informative: When the source value is unknown, the runtime returns `Missing` to trigger the associated question. (See REQ-HIPP-MEAN-002.)

Requirements:

#### REQ-HIPP-MEAN-001 — `meaning of <value>` expressions evaluate using the value's meaning assessments

**Traces to:** SREQ-HIPP-012

`meaning of <value>` expressions evaluate using the value's meaning assessments.

#### REQ-HIPP-MEAN-002 — Meaning evaluation returns `Missing` when the source value is unknown and askable

**Traces to:** SREQ-HIPP-012

meaning evaluation returns `Missing` when the source value is unknown and askable.

#### REQ-HIPP-MEAN-003 — Meaning evaluation supports nested assessments within meaning cases

**Traces to:** SREQ-HIPP-012

meaning evaluation supports nested assessments within meaning cases.

## 6. Examples by Feature

### 6.1. Visual Analogue Scale (VAS)

```hippocrates
<point> is a unit:
    plural is <points>

<pain level> is a number:
    valid values:
        0 <points> ... 10 <points>
    question:
        ask "How severe is your pain?":
            question style is visual analogue scale:
                best value is 0 <points>.
                text for best value is "No pain".
                worst value is 10 <points>.
                text for worst value is "Worst pain imaginable".
```

### 6.2. Filtered Calculations with Timeframes

```hippocrates
<dose> is a unit:
    plural is <doses>

<best inhalation period> is a period:
    timeframe:
        between Monday ... Sunday; 07:00 ... 09:00

<inhaler used in past 5 days> is a number:
    valid values:
        0 <doses> ... 1000 <doses>
    calculation:
        timeframe for analysis is between 5 days ago ... now:
            <inhaler used in past 5 days> = count of <inhaler used> is <Yes>.

<inhaler used in past 5 days on time> is a number:
    valid values:
        0 <doses> ... 1000 <doses>
    calculation:
        timeframe for analysis is between 5 days ago ... now during <best inhalation period>:
            <inhaler used in past 5 days on time> = count of <inhaler used> is <Yes>.
```

### 6.3. Handling Insufficient Data

```hippocrates
<point> is a unit:
    plural is <points>

<weekly average> is a number:
    valid values:
        0 <points> ... 10 <points>
    calculation:
        timeframe for analysis is 7 days ago ... now:
            <weekly average> = average of <daily pain> over 7 days.

before plan:
    assess <weekly average>:
        Not enough data:
            information "Please continue tracking pain for a full week.".
        0 <points> ... 5 <points>:
            information "Your pain levels are within range this week.".
        6 <points> ... 10 <points>:
            warning "Your pain levels are high this week.".
```

### 6.4. Message Expiration

```hippocrates
urgent warning to <patient> "Take your medication now":
    message expires after 15 minutes.
```

### 6.5. Question Expiration and Reminders

```hippocrates
<point> is a unit:
    plural is <points>

<pain score> is a number:
    valid values:
        0 <points> ... 10 <points>
    question:
        ask "How severe is your pain?":
            question expires after 1 day:
                information "We still need your answer for today's pain score.".
```

### 6.6. Validity Timeframe (Reuse)

```hippocrates
<body temperature> is a number:
    valid values:
        35.0 °C ... 42.0 °C
    reuse:
        reuse period of value is 1 hour.

before plan:
    ask <body temperature>.
```

### 6.7. Date/Time Intake Questions

```hippocrates
<time of injury> is a date/time:
    valid values:
        10 days ago ... now.
    question:
        ask "When did the injury happen?".

<hours since meal> is a number:
    unit is hours.
    valid values:
        0 hours ... 24 hours.
    question:
        ask "How many hours ago did you eat?".

<intake> is a plan:
    before plan:
        ask <time of injury>.
        ask <hours since meal>.
```

## 7. Requirements Index

- REQ-HIPP-LANG-001: identifiers must use angle brackets.
- REQ-HIPP-LANG-002: string literals must not contain angle brackets.
- REQ-HIPP-LANG-003: comparison operators are not supported; use ranges.
- REQ-HIPP-LANG-004: block openings require a newline and indented block.
- REQ-HIPP-BASIC-001: time indications parse for now, weekday, and time-of-day.
- REQ-HIPP-BASIC-002: relative time expressions from now parse.
- REQ-HIPP-BASIC-003: inline ':' forms are only allowed where explicitly shown.
- REQ-HIPP-BASIC-004: date/time literals parse for date and date-time forms.
- REQ-HIPP-UNITS-001: custom unit pluralization canonicalizes values.
- REQ-HIPP-UNITS-002: standard units work in calculations.
- REQ-HIPP-UNITS-003: custom unit abbreviations canonicalize values.
- REQ-HIPP-UNITS-004: custom unit quantities parse with definitions.
- REQ-HIPP-UNITS-005: numeric literals must include units.
- REQ-HIPP-PROG-001: multi-definition fixtures parse core definitions.
- REQ-HIPP-VALUE-001: value definitions parse from fixtures.
- REQ-HIPP-VALUE-002: value type variants parse correctly.
- REQ-HIPP-VALUE-003: unit properties parse in numeric values.
- REQ-HIPP-VALUE-004: value timeframe properties parse.
- REQ-HIPP-VALUE-005: inheritance properties parse with overrides.
- REQ-HIPP-VALUE-006: documentation properties parse in inline and block forms.
- REQ-HIPP-VALUE-007: custom properties parse as generic properties.
- REQ-HIPP-VALUE-008: date/time value types parse correctly.
- REQ-HIPP-VALUE-009: meaning assessments are only allowed in value definition blocks.
- REQ-HIPP-VALUE-010: meaning properties require an explicit target identifier.
- REQ-HIPP-VALUE-011: meaning properties must declare valid meanings.
- REQ-HIPP-VALUE-012: meaning labels must be identifiers (angle brackets).
- REQ-HIPP-VALUE-013: enumeration valid values are identifiers (angle brackets).
- REQ-HIPP-PLAN-001: period definitions parse by name.
- REQ-HIPP-PLAN-002: period timeframe lines parse with range selectors.
- REQ-HIPP-STMT-001: timeframe blocks parse with nested statements.
- REQ-HIPP-STMT-002: timeframe selectors require a start and end.
- REQ-HIPP-STMT-003: timeframe selector identifiers must refer to defined periods.
- REQ-HIPP-STMT-004: statements inside blocks must terminate with a period.
- REQ-HIPP-STMT-005: blocks must be introduced with a colon.
- REQ-HIPP-ACT-001: question configuration parses and validates references.
- REQ-HIPP-ACT-002: message expiration attaches to information, warning, and urgent warning actions.
- REQ-HIPP-ACT-003: question modifiers parse (validate/type/style/expire).
- REQ-HIPP-ACT-004: validate answer within parsing attaches to ask blocks.
- REQ-HIPP-ACT-005: listen/send/start/simple command actions parse.
- REQ-HIPP-ACT-006: question expiration blocks parse with reminder statements.
- REQ-HIPP-ACT-007: question expiration supports until event triggers.
- REQ-HIPP-ACT-008: information, warning, and urgent warning are accepted as message action keywords.
- REQ-HIPP-ACT-009: message actions accept semicolon-separated addressee lists.
- REQ-HIPP-ACT-010: `after plan:` block syntax and semantics. The `after plan:` block is a plan block that executes its statements exactly once when the event loop terminates.
- REQ-HIPP-EVT-001: event triggers parse for change/start/periodic.
- REQ-HIPP-EVT-002: event blocks attach statements to triggers.
- REQ-HIPP-EVT-003: scheduler computes next occurrence for periods.
- REQ-HIPP-EVT-004: periodic triggers parse duration and offsets.
- REQ-HIPP-EVT-005: periodic triggers accept an optional `at <time_literal>` clause to pin execution to a specific time of day.
- REQ-HIPP-EVT-006: periodic triggers using a named period identifier with a `for` duration fire at every occurrence of that period within the duration window.
- REQ-HIPP-COMM-001: addressee groups and contact logic parse.
- REQ-HIPP-COMM-002: contact details and sequence ordering parse.
- REQ-HIPP-MED-001: drug definition validation rejects undefined units.
- REQ-HIPP-MED-002: drug interaction properties parse.
- REQ-HIPP-MED-003: dosage safety and administration rules parse.
- REQ-HIPP-CTX-001: context definitions parse timeframe/data/value filter items.
- REQ-HIPP-CTX-002: context blocks parse data/value filters and nested statements.
- REQ-HIPP-CTX-003: context for analysis executes with scoped timeframe.
- REQ-HIPP-EXPR-001: statistical function expressions parse in assignments.
- REQ-HIPP-EXPR-002: timeframe filtering applies to statistical evaluations.
- REQ-HIPP-EXPR-003: timeframe variants resolve counts over different windows.
- REQ-HIPP-EXPR-004: trend analysis evaluates statistical trends over timeframes.
- REQ-HIPP-EXPR-005: statistical functions require an analysis timeframe context.
- REQ-HIPP-EXPR-006: date diff expressions parse.
- REQ-HIPP-EXPR-007: meaning-of expressions parse.
- REQ-HIPP-CORE-001: built-in units cannot be redefined.
- REQ-HIPP-CORE-002: unit conversions are supported within compatible groups.
- REQ-HIPP-CORE-003: calculations and assignments require matching units and precision.
- REQ-HIPP-CORE-005: parse errors shall include human-readable descriptions, not raw PEG rule names.
- REQ-HIPP-REQP-001: numeric valid values require units.
- REQ-HIPP-REQP-002: assessment ranges require units.
- REQ-HIPP-REQP-003: numeric definitions require units.
- REQ-HIPP-REQP-004: ask requires a question property on the value.
- REQ-HIPP-REQP-005: unit requirement errors report line numbers.
- REQ-HIPP-REQP-006: numbers and enumerations must define valid values.
- REQ-HIPP-REQP-007: valid value ranges must not overlap.
- REQ-HIPP-FLOW-001: values cannot be used before assignment.
- REQ-HIPP-FLOW-002: calculation properties do not seed values.
- REQ-HIPP-FLOW-003: statistical functions do not require local initialization.
- REQ-HIPP-FLOW-004: listen for and context data initialize values.
- REQ-HIPP-FLOW-005: meaning-of expressions require an askable value when not initialized.
- REQ-HIPP-COVER-001: meaning ranges must cover valid values (integer gaps).
- REQ-HIPP-COVER-002: meaning ranges must cover valid values (float gaps).
- REQ-HIPP-COVER-003: disjoint valid ranges are allowed when fully covered.
- REQ-HIPP-COVER-004: overlapping numeric assessment ranges are invalid.
- REQ-HIPP-COVER-005: duplicate enumeration cases are invalid.
- REQ-HIPP-COVER-006: gap detection reports missing integer spans.
- REQ-HIPP-COVER-007: gap detection reports missing float spans.
- REQ-HIPP-COVER-008: coverage gaps respect precision for float and integer ranges.
- REQ-HIPP-COVER-009: overlapping ranges are rejected.
- REQ-HIPP-COVER-010: missing coverage yields a validation error.
- REQ-HIPP-COVER-011: trend assessments require full coverage.
- REQ-HIPP-COVER-012: numeric valid value ranges use consistent precision across bounds and intervals.
- REQ-HIPP-COVER-013: valid meanings must be fully used across meaning assessments.
- REQ-HIPP-COVER-014: meaning labels must be drawn from declared valid meanings.
- REQ-HIPP-RANGE-001: interval math supports range compliance checks.
- REQ-HIPP-RANGE-002: assignment range compliance fails when out of bounds.
- REQ-HIPP-SUFF-001: timeframe calculations require Not enough data handling.
- REQ-HIPP-SUFF-002: Not enough data handling satisfies sufficiency.
- REQ-HIPP-SUFF-003: runtime evaluation returns NotEnoughData when history is insufficient.
- REQ-HIPP-SUFF-004: Not enough data is only allowed for statistical assessments.
- REQ-HIPP-DTIME-001: date/time valid value ranges evaluate using date/time and time-of-day semantics.
- REQ-HIPP-DTIME-002: date diff expressions evaluate to quantities in requested units.
- REQ-HIPP-EXEC-001: runtime executes assignments and actions in order.
- REQ-HIPP-EXEC-002: reuse timeframes prevent re-asking within the validity window.
- REQ-HIPP-EXEC-003: runtime emits a warning when a message action executes without a message callback.
- REQ-HIPP-EXEC-004: simulation mode executes plans at accelerated speed without real-time delays.
- REQ-HIPP-EXEC-005: when a periodic trigger includes an `at` time, the first execution is scheduled at that time on the first eligible day, and subsequent executions recur at that same time each interval.
- REQ-HIPP-VALID-001: full-plan validation passes for a complete plan.
- REQ-HIPP-VALID-002: the validator shall detect references to undeclared variables, addressees, units, and periods, listing available definitions.
- REQ-HIPP-VALID-003: validation errors shall include a `suggestion` field with an actionable fix description.
- REQ-HIPP-INPUT-001: numeric answers must respect the decimal precision implied by valid values.
- REQ-HIPP-MEAN-001: meaning-of expressions evaluate using meaning assessments.
- REQ-HIPP-MEAN-002: meaning evaluation returns Missing for unknown askable values.
- REQ-HIPP-MEAN-003: meaning evaluation supports nested assessments.

---

## Revision History

| Rev  | Date       | Author         | Description                                              |
|------|------------|----------------|----------------------------------------------------------|
| 1.0  | 2026-03-20 | (auto-adopted) | Initial adoption of Language Specification v1.0 as SYRS-01. V-Model preamble and traceability matrix added. |
| 1.1  | 2026-03-20 | (auto-adopted) | Added REQ-HIPP-EXEC-004 (simulation mode). |
| 1.2  | 2026-03-23 | V-Model | Added REQ-HIPP-EVT-005 (at time clause), REQ-HIPP-EVT-006 (period repetition), REQ-HIPP-EXEC-005 (time-pinned scheduling). Updated EBNF grammar. |
| 1.3  | 2026-03-23 | V-Model | Added REQ-HIPP-ACT-010 (`after plan:` block). Updated EBNF: `plan_block` gains `after_plan_block` alternative. |
| 1.4  | 2026-03-23 | V-Model | Added REQ-HIPP-EVT-007 (bare unit and ordinal trigger syntax). Updated EBNF: added `ordinal` and `bare_unit` rules, updated `event_trigger`. |
| 1.5  | 2026-03-23 | V-Model | Added REQ-HIPP-CORE-005 (human-readable parse errors), REQ-HIPP-VALID-002 (undefined reference detection), REQ-HIPP-VALID-003 (suggested fixes). All trace to SREQ-HIPP-037. |
| 1.6  | 2026-04-19 | V-Model | Moved section-level informative prose (bullet overviews, EBNF grammar, narrative paragraphs) from the tail of the last H4 in each section to before the `Requirements:` label, so each REQ-HIPP-\* heading renders with only its own body. |
