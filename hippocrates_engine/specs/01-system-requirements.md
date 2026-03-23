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
| §2 Language Principles             | STKR-02 (Readability), STKR-30 (No Comparison Operators), STKR-13 (Contextual Statements) |
| §3.1-3.3 Grammar                   | STKR-02 (Readability), STKR-11 (Readability Over Defaults)   |
| §3.2 Units                         | STKR-18 (Medical Domain), STKR-31 (Unit Discipline)          |
| §3.4 Values                        | STKR-12 (Meaningful Values), STKR-15 (Value Constraints)     |
| §3.5 Periods/Plans                 | STKR-05 (Event-Driven), STKR-20 (Plan Autonomy)              |
| §3.6 Statements                    | STKR-10 (Completeness), STKR-13 (Contextual Statements)      |
| §3.7 Actions/Questions             | STKR-01 (Care Plan Execution), STKR-05 (Event-Driven)        |
| §3.8 Events/Timing                 | STKR-05 (Event-Driven)                                       |
| §3.9 Communication                 | STKR-01 (Care Plan Execution), STKR-14 (Separation)          |
| §3.10 Medication                   | STKR-01 (Care Plan Execution), STKR-18 (Medical Domain)      |
| §3.11 Data Contexts                | STKR-13 (Contextual Statements)                              |
| §3.12 Expressions/Stats            | STKR-35 (Data Sufficiency)                                   |
| §4.1 Units/Conversion              | STKR-31 (Unit Discipline), STKR-18 (Medical Domain)          |
| §4.2 Required Properties           | STKR-15 (Value Constraints), STKR-11 (Readability Over Defaults) |
| §4.3 Data Flow                     | STKR-33 (Data Flow Validation)                               |
| §4.4 Coverage                      | STKR-10 (Completeness), STKR-32 (Exhaustive Coverage)        |
| §4.5 Range Compliance              | STKR-15 (Value Constraints)                                  |
| §4.6 Data Sufficiency              | STKR-35 (Data Sufficiency)                                   |
| §4.7 Date/Time                     | STKR-18 (Medical Domain)                                     |
| §5 Execution                       | STKR-01 (Care Plan Execution), STKR-05 (Event-Driven)        |
| §5.1 Validation                    | STKR-04 (Validation Without Execution)                       |
| §5.2 Input Validation              | STKR-34 (Input Precision)                                    |
| §5.3 Meaning Evaluation            | STKR-12 (Meaningful Values)                                  |

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

Requirements:
- REQ-2-01: identifiers must use angle brackets.
- REQ-2-02: string literals must not contain angle brackets.
- REQ-2-03: comparison operators are not supported; use ranges.
- REQ-2-04: block openings require a newline and indented block.




* **Natural Language Syntax**: Statements mimic English sentences to ensure readability by medical professionals.
* **Type Safety & Units**: Informative — numeric literals are expressed as quantities with units (built-in or custom), e.g., `10 mg` or `7 days`. (See REQ-3.2-05.)
* **Contextual Execution**: Scripts execute within a specific context (Patient, Timeframe).
* **Event-Driven**: The core runtime is an event loop reacting to time, value changes, and external triggers.
* **Completeness**: A plan describes a self-contained logic for a single subject (the Patient).
* **Angle-Bracket Identifiers**: Informative — identifiers are written as `<...>`. (See REQ-2-01.)
* **Indented Blocks**: Informative — any `:` that opens a block is followed by a newline and indented block. (See REQ-2-04.)
* **Block Syntax**: Informative — a block is introduced with a trailing `:` and statements inside the block end with a `.`. (See REQ-3.6-04, REQ-3.6-05.)
* **No Comparison Operators**: Informative — use ranges (`min ... max`) instead of `<`, `>`, `<=`, `>=`. (See REQ-2-03.)

## 3. Formal Grammar (EBNF)

The following grammar defines the syntax of Hippocrates. Indentation is significant and is converted into explicit `INDENT`/`DEDENT` tokens by the parser.

### 3.1. Basic Elements

Requirements:
- REQ-3.1-01: time indications parse for now, weekday, and time-of-day.
- REQ-3.1-02: relative time expressions from now parse.
- REQ-3.1-03: inline ':' forms are only allowed where explicitly shown.
- REQ-3.1-04: date/time literals parse for date and date-time forms.




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

Informative: Identifiers are angle-bracketed. When a rule introduces an indented block (`:` followed by `indent`/`dedent`), a newline is used. Inline `:` forms appear only where explicitly shown (e.g., `documentation` strings). (See REQ-2-01, REQ-2-04, REQ-3.1-03.)
Informative: A block is introduced with a trailing `:` and statements inside the block end with a `.`. (See REQ-3.6-04, REQ-3.6-05.)

### 3.2. Units and Quantities

Requirements:
- REQ-3.2-01: custom unit pluralization canonicalizes values.
- REQ-3.2-02: standard units work in calculations.
- REQ-3.2-03: custom unit abbreviations canonicalize values.
- REQ-3.2-04: custom unit quantities parse with definitions.
- REQ-3.2-05: numeric literals must include units.




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

Informative: Numeric literals in user scripts are expressed as quantities with units; unitless numbers are invalid. (See REQ-3.2-05.)
Informative: Built-in units are reserved and cannot be redefined or aliased in unit definitions. (See REQ-4.1-01.)

Informative: Precision is defined by the number of digits after the decimal point (e.g., `0` → precision 0, `0.0` → precision 1, `0.00` → precision 2). Each numeric valid value range uses the same precision for both bounds, and all ranges in a `valid values` block share the same precision. Integer ranges (e.g., `0 <points> ... 10 <points>`) use step size 1; decimal ranges (e.g., `0.0 mg ... 10.0 mg`) use the declared precision (step size `10^-precision`). (See REQ-4.4-08, REQ-4.4-12.)

### 3.3. Program Structure

Requirements:
- REQ-3.3-01: multi-definition fixtures parse core definitions.




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

### 3.4. Values

Requirements:
- REQ-3.4-01: value definitions parse from fixtures.
- REQ-3.4-02: value type variants parse correctly.
- REQ-3.4-03: unit properties parse in numeric values.
- REQ-3.4-04: value timeframe properties parse.
- REQ-3.4-05: inheritance properties parse with overrides.
- REQ-3.4-06: documentation properties parse in inline and block forms.
- REQ-3.4-07: custom properties parse as generic properties.
- REQ-3.4-08: date/time value types parse correctly.
- REQ-3.4-09: meaning assessments are only allowed in value definition blocks.
- REQ-3.4-10: meaning properties require an explicit target identifier.
- REQ-3.4-11: meaning properties must declare valid meanings.
- REQ-3.4-12: meaning labels must be identifiers (angle brackets).
- REQ-3.4-13: enumeration valid values are identifiers (angle brackets).




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

Informative: `meaning of <value>:` blocks must include the target identifier; the shorthand `meaning:` is not part of the language. (See REQ-3.4-10.)
Informative: `meaning of <value>:` blocks must declare `valid meanings:` before meaning assessments. (See REQ-3.4-11.)
Informative: Meaning labels and `valid meanings` entries are identifiers (angle brackets), not string literals. (See REQ-3.4-12.)
Informative: `meaning of <value>:` blocks and `assess meaning of <value>` blocks are value properties only; they are not valid statements inside plans or other block contexts. (See REQ-3.4-09.)

### 3.5. Periods and Plans

Requirements:
- REQ-3.5-01: period definitions parse by name.
- REQ-3.5-02: period timeframe lines parse with range selectors.




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

### 3.6. Statements, Assessments, and Ranges

Requirements:
- REQ-3.6-01: timeframe blocks parse with nested statements.
- REQ-3.6-02: timeframe selectors require a start and end.
- REQ-3.6-03: timeframe selector identifiers must refer to defined periods.
- REQ-3.6-04: statements inside blocks must terminate with a period.
- REQ-3.6-05: blocks must be introduced with a colon.




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

Informative: Timeframe selectors include a start and an end; single time indications (e.g., `now`) are not valid timeframes. When a timeframe selector uses an identifier, it refers to a period definition that provides the underlying start/end bounds. (See REQ-3.6-02, REQ-3.6-03.)

constraint = expression, constraint_operator, range_selector, ".";
constraint_operator = "is" | "during" | "after";

block = { statement };

context_block = "context", [ " for analysis" ], ":", newline, indent, { context_item | statement }, dedent;

Informative: Statements inside indented blocks terminate with a period (`.`). Blocks are introduced with a colon (`:`). (See REQ-3.6-04, REQ-3.6-05.)

timeframe_block =
    "timeframe", [ " for analysis" ],
    constraint_operator, timeframe_selector, { constraint_operator, timeframe_selector },
    ":", newline, indent, { statement }, dedent;
```

### 3.7. Actions and Questions

Requirements:
- REQ-3.7-01: question configuration parses and validates references.
- REQ-3.7-02: message expiration attaches to information, warning, and urgent warning actions.
- REQ-3.7-03: question modifiers parse (validate/type/style/expire).
- REQ-3.7-04: validate answer within parsing attaches to ask blocks.
- REQ-3.7-05: listen/send/start/simple command actions parse.
- REQ-3.7-06: question expiration blocks parse with reminder statements.
- REQ-3.7-07: question expiration supports until event triggers.
- REQ-3.7-08: information, warning, and urgent warning are accepted as message action keywords.
- REQ-3.7-09: message actions accept semicolon-separated addressee lists.
- REQ-3.7-10: `after plan:` block syntax and semantics. The `after plan:` block is a plan block that executes its statements exactly once when the event loop terminates (all triggers exhausted or simulation time limit reached).




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


### 3.8. Events and Timing

Requirements:
- REQ-3.8-01: event triggers parse for change/start/periodic.
- REQ-3.8-02: event blocks attach statements to triggers.
- REQ-3.8-03: scheduler computes next occurrence for periods.
- REQ-3.8-04: periodic triggers parse duration and offsets.
- REQ-3.8-05: periodic triggers accept an optional `at <time_literal>` clause to pin execution to a specific time of day.
- REQ-3.8-06: periodic triggers using a named period identifier with a `for` duration fire at every occurrence of that period within the duration window.
- REQ-3.8-07: the grammar shall accept bare unit names (`every day`, `every week`) as shorthand for `every 1 day`, `every 1 week`. The grammar shall accept ordinals (`every second day`, `every third week`, `every other day`) as shorthand for the corresponding numeric interval. Supported ordinals: other/second (2), third (3), fourth (4), fifth (5), sixth (6), seventh (7), eighth (8), ninth (9), tenth (10).


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


### 3.9. Communication & Actors

Requirements:
- REQ-3.9-01: addressee groups and contact logic parse.
- REQ-3.9-02: contact details and sequence ordering parse.




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


### 3.10. Medication

Requirements:
- REQ-3.10-01: drug definition validation rejects undefined units.
- REQ-3.10-02: drug interaction properties parse.
- REQ-3.10-03: dosage safety and administration rules parse.




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


### 3.11. Data Contexts

Requirements:
- REQ-3.11-01: context definitions parse timeframe/data/value filter items.
- REQ-3.11-02: context blocks parse data/value filters and nested statements.
- REQ-3.11-03: context for analysis executes with scoped timeframe.




```ebnf
context_definition = "context:", newline, indent, { context_item }, dedent;

context_item =
    "timeframe:", timeframe_selector |
    "data:", identifier |
    "value filter:", assessment_case;
```

### 3.12. Expressions and Statistical Analysis

Requirements:
- REQ-3.12-01: statistical function expressions parse in assignments.
- REQ-3.12-02: timeframe filtering applies to statistical evaluations.
- REQ-3.12-03: timeframe variants resolve counts over different windows.
- REQ-3.12-04: trend analysis evaluates statistical trends over timeframes.
- REQ-3.12-05: statistical functions require an analysis timeframe context.
- REQ-3.12-06: date diff expressions parse.
- REQ-3.12-07: meaning-of expressions parse.




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

Informative: Statistical functions are evaluated within an analysis timeframe. Use a `timeframe for analysis` block or a `context for analysis` block that provides a `timeframe:` item. (See REQ-3.12-05.)

## 4. Semantics and Type System

### 4.1. Core Unit Groups and Conversion

Requirements:
- REQ-4.1-01: built-in units cannot be redefined.
- REQ-4.1-02: unit conversions are supported within compatible groups.
- REQ-4.1-03: calculations and assignments require matching units and precision.




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

Informative: The runtime checks that values compared have compatible units (belong to the same group). Calculations and assignments require exact unit matches and the same numeric precision as the target value. (See REQ-4.1-02, REQ-4.1-03.)

#### Unit Normalization

Informative: For custom units (e.g., `points`, `tablets`), pluralization and abbreviations are defined explicitly. Without a definition, `10 <point>` and `10 <points>` are treated as different units. (See REQ-3.2-01, REQ-3.2-03.)

```hippocrates
<point> is a unit:
    plural is <points>
    abbreviation is "pts"
```

### 4.2. Required Properties

Requirements:
- REQ-4.2-01: numeric valid values require units.
- REQ-4.2-02: assessment ranges require units.
- REQ-4.2-03: numeric definitions require units.
- REQ-4.2-04: ask requires a question property on the value.
- REQ-4.2-05: unit requirement errors report line numbers.
- REQ-4.2-06: numbers and enumerations must define valid values.
- REQ-4.2-07: valid value ranges must not overlap.




* **Numbers and Enumerations**: Informative — `valid values` are defined. (See REQ-4.2-06.)
* **Numbers**: Informative — a unit is defined (via `unit is ...` or by using quantities in `valid values`). (See REQ-4.2-01, REQ-4.2-03.)
* **Asking**: Informative — `ask` is only valid when a value has a `question` property. (See REQ-4.2-04.)
* **Valid Values**: Informative — valid value ranges are non-overlapping (including numeric, date/time, and time-of-day ranges); use disjoint ranges when multiple intervals are needed. (See REQ-4.2-07.)
* **Enumerations**: Informative — enumeration valid values are identifiers (angle brackets), not string literals. (See REQ-3.4-13.)

### 4.3. Data Flow and Validity

Requirements:
- REQ-4.3-01: values cannot be used before assignment.
- REQ-4.3-02: calculation properties do not seed values.
- REQ-4.3-03: statistical functions do not require local initialization.
- REQ-4.3-04: listen for and context data initialize values.
- REQ-4.3-05: meaning-of expressions require an askable value when not initialized.




* Informative — a value is not used before it has valid content. (See REQ-4.3-01.)
* Informative — values gain valid content by being assigned, asked, or provided by `listen for` or `context data:`. (See REQ-4.3-04.)
* Informative — calculation properties describe how a value is derived but do not implicitly seed it; plans assign or ask before use. (See REQ-4.3-02.)
* Informative — statistical functions read history and do not require local initialization of the referenced value. (See REQ-4.3-03.)
* Informative — `meaning of <value>` is valid only if the value is already initialized or has a `question` property so the runtime can ask when missing. (See REQ-4.3-05.)

### 4.4. Assessment Coverage

Requirements:
- REQ-4.4-01: meaning ranges must cover valid values (integer gaps).
- REQ-4.4-02: meaning ranges must cover valid values (float gaps).
- REQ-4.4-03: disjoint valid ranges are allowed when fully covered.
- REQ-4.4-04: overlapping numeric assessment ranges are invalid.
- REQ-4.4-05: duplicate enumeration cases are invalid.
- REQ-4.4-06: gap detection reports missing integer spans.
- REQ-4.4-07: gap detection reports missing float spans.
- REQ-4.4-08: coverage gaps respect precision for float and integer ranges.
- REQ-4.4-09: overlapping ranges are rejected.
- REQ-4.4-10: missing coverage yields a validation error.
- REQ-4.4-11: trend assessments require full coverage.
- REQ-4.4-12: numeric valid value ranges use consistent precision across bounds and intervals.
- REQ-4.4-13: valid meanings must be fully used across meaning assessments.
- REQ-4.4-14: meaning labels must be drawn from declared valid meanings.




Meaning assessments are declared as `meaning of <value>:` and must include `valid meanings:`; they may include `assess meaning of <value>` blocks. Meaning labels are identifiers (angle brackets). Within a meaning case, a bare identifier statement (e.g., `<light>.`) assigns that meaning label, equivalent to `meaning of value = <light>.`.

* Informative — `assess` blocks, `meaning` cases, and assessments over statistical results fully cover the valid range of the target/output. (See REQ-4.4-01, REQ-4.4-02, REQ-4.4-03.)
* Informative — for enumerations, all valid values are covered. (See REQ-4.4-05.)
* Informative — for `trend of <value>`, all cases (`"increase"`, `"decrease"`, `"stable"`) are covered. (See REQ-4.4-11.)
* Informative — when `valid meanings` are declared, meaning assessments use all declared labels and no others. (See REQ-4.4-13, REQ-4.4-14.)

### 4.5. Range Compliance (Pre-Run Validation)

Requirements:
- REQ-4.5-01: interval math supports range compliance checks.
- REQ-4.5-02: assignment range compliance fails when out of bounds.




Informative: Before execution, the runtime validates that calculated and assigned values remain within their declared ranges. If the computed range can exceed the valid values, validation fails. (See REQ-4.5-01, REQ-4.5-02.)

### 4.6. Data Sufficiency

Requirements:
- REQ-4.6-01: timeframe calculations require Not enough data handling.
- REQ-4.6-02: Not enough data handling satisfies sufficiency.
- REQ-4.6-03: runtime evaluation returns NotEnoughData when history is insufficient.
- REQ-4.6-04: Not enough data is only allowed for statistical assessments.




Informative: Calculations involving history use `Not enough data` when the available history is shorter than the requested timeframe. This is handled explicitly in assessments. (See REQ-4.6-01, REQ-4.6-02, REQ-4.6-03.)
Informative: `Not enough data` assessments are only used when the assessed target is derived from statistical functions. (See REQ-4.6-04.)

### 4.7 Date/Time Semantics

Requirements:
- REQ-4.7-01: date/time valid value ranges evaluate using date/time and time-of-day semantics.
- REQ-4.7-02: date diff expressions evaluate to quantities in requested units.



* Informative — date/time ranges compare full date-time values; time-only ranges compare time-of-day and may wrap over midnight. (See REQ-4.7-01.)
* Informative — date diffs return whole calendar months/years and fractional day/hour/minute quantities based on elapsed time. (See REQ-4.7-02.)

## 5. Execution Model

Requirements:
- REQ-5-01: runtime executes assignments and actions in order.
- REQ-5-02: reuse timeframes prevent re-asking within the validity window.
- REQ-5-03: runtime emits a warning when a message action executes without a message callback.
- REQ-5-04: simulation mode executes plans at accelerated speed without real-time delays.
- REQ-5-05: when a periodic trigger includes an `at` time, the first execution is scheduled at that time on the first eligible day, and subsequent executions recur at that same time each interval.




The Hippocrates Runtime functions as a **State Machine**.

1. **Load**: Parse script, build internal dependencies graph (DAG).
2. **Init**: Initialize all values to `unknown` or default; restore state from persistence.
3. **Loop**:
    * Check **Timers**: Are there any temporal events (`every 1 day`, `every Monday`)? -> Trigger Event.
    * Check **Inputs**: Did an external API update a value? -> Trigger `change of` Event.
    * Evaluate **Rules**: If an Event triggered, execute the associated `block`.
    * **Side Effects**: Execute `information`, `warning`, `urgent warning`, `ask`, or `send information` commands via API callbacks.
    * Informative — message delivery uses the message callback; if it is not provided, the runtime logs a warning. (See REQ-5-03.)

### 5.1 Validation Logic

Requirements:
- REQ-5.1-01: full-plan validation passes for a complete plan.




Before execution, the runtime validates that:

1. All `assess` blocks and `meaning` cases cover the complete valid range of the target value.
2. No values are used before they are initialized or asked.
3. All referenced variables and units are compatible.

### 5.2 Input Validation

Requirements:
- REQ-5.2-01: numeric answers must respect the decimal precision implied by valid values.



Informative: Integer valid values (e.g., `0 hours ... 24 hours`) reject fractional inputs; decimal ranges allow their declared precision. (See REQ-5.2-01.)

### 5.3 Meaning Evaluation

Requirements:
- REQ-5.3-01: `meaning of <value>` expressions evaluate using the value's meaning assessments.
- REQ-5.3-02: meaning evaluation returns `Missing` when the source value is unknown and askable.
- REQ-5.3-03: meaning evaluation supports nested assessments within meaning cases.



Informative: Meaning evaluation selects the first matching meaning case for the value and returns the assigned meaning label. Nested `assess` blocks are evaluated to resolve the final meaning. (See REQ-5.3-01, REQ-5.3-03.)
Informative: When the source value is unknown, the runtime returns `Missing` to trigger the associated question. (See REQ-5.3-02.)

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

- REQ-2-01: identifiers must use angle brackets.
- REQ-2-02: string literals must not contain angle brackets.
- REQ-2-03: comparison operators are not supported; use ranges.
- REQ-2-04: block openings require a newline and indented block.
- REQ-3.1-01: time indications parse for now, weekday, and time-of-day.
- REQ-3.1-02: relative time expressions from now parse.
- REQ-3.1-03: inline ':' forms are only allowed where explicitly shown.
- REQ-3.1-04: date/time literals parse for date and date-time forms.
- REQ-3.2-01: custom unit pluralization canonicalizes values.
- REQ-3.2-02: standard units work in calculations.
- REQ-3.2-03: custom unit abbreviations canonicalize values.
- REQ-3.2-04: custom unit quantities parse with definitions.
- REQ-3.2-05: numeric literals must include units.
- REQ-3.3-01: multi-definition fixtures parse core definitions.
- REQ-3.4-01: value definitions parse from fixtures.
- REQ-3.4-02: value type variants parse correctly.
- REQ-3.4-03: unit properties parse in numeric values.
- REQ-3.4-04: value timeframe properties parse.
- REQ-3.4-05: inheritance properties parse with overrides.
- REQ-3.4-06: documentation properties parse in inline and block forms.
- REQ-3.4-07: custom properties parse as generic properties.
- REQ-3.4-08: date/time value types parse correctly.
- REQ-3.4-09: meaning assessments are only allowed in value definition blocks.
- REQ-3.4-10: meaning properties require an explicit target identifier.
- REQ-3.4-11: meaning properties must declare valid meanings.
- REQ-3.4-12: meaning labels must be identifiers (angle brackets).
- REQ-3.4-13: enumeration valid values are identifiers (angle brackets).
- REQ-3.5-01: period definitions parse by name.
- REQ-3.5-02: period timeframe lines parse with range selectors.
- REQ-3.6-01: timeframe blocks parse with nested statements.
- REQ-3.6-02: timeframe selectors require a start and end.
- REQ-3.6-03: timeframe selector identifiers must refer to defined periods.
- REQ-3.6-04: statements inside blocks must terminate with a period.
- REQ-3.6-05: blocks must be introduced with a colon.
- REQ-3.7-01: question configuration parses and validates references.
- REQ-3.7-02: message expiration attaches to information, warning, and urgent warning actions.
- REQ-3.7-03: question modifiers parse (validate/type/style/expire).
- REQ-3.7-04: validate answer within parsing attaches to ask blocks.
- REQ-3.7-05: listen/send/start/simple command actions parse.
- REQ-3.7-06: question expiration blocks parse with reminder statements.
- REQ-3.7-07: question expiration supports until event triggers.
- REQ-3.7-08: information, warning, and urgent warning are accepted as message action keywords.
- REQ-3.7-09: message actions accept semicolon-separated addressee lists.
- REQ-3.7-10: `after plan:` block syntax and semantics. The `after plan:` block is a plan block that executes its statements exactly once when the event loop terminates.
- REQ-3.8-01: event triggers parse for change/start/periodic.
- REQ-3.8-02: event blocks attach statements to triggers.
- REQ-3.8-03: scheduler computes next occurrence for periods.
- REQ-3.8-04: periodic triggers parse duration and offsets.
- REQ-3.8-05: periodic triggers accept an optional `at <time_literal>` clause to pin execution to a specific time of day.
- REQ-3.8-06: periodic triggers using a named period identifier with a `for` duration fire at every occurrence of that period within the duration window.
- REQ-3.9-01: addressee groups and contact logic parse.
- REQ-3.9-02: contact details and sequence ordering parse.
- REQ-3.10-01: drug definition validation rejects undefined units.
- REQ-3.10-02: drug interaction properties parse.
- REQ-3.10-03: dosage safety and administration rules parse.
- REQ-3.11-01: context definitions parse timeframe/data/value filter items.
- REQ-3.11-02: context blocks parse data/value filters and nested statements.
- REQ-3.11-03: context for analysis executes with scoped timeframe.
- REQ-3.12-01: statistical function expressions parse in assignments.
- REQ-3.12-02: timeframe filtering applies to statistical evaluations.
- REQ-3.12-03: timeframe variants resolve counts over different windows.
- REQ-3.12-04: trend analysis evaluates statistical trends over timeframes.
- REQ-3.12-05: statistical functions require an analysis timeframe context.
- REQ-3.12-06: date diff expressions parse.
- REQ-3.12-07: meaning-of expressions parse.
- REQ-4.1-01: built-in units cannot be redefined.
- REQ-4.1-02: unit conversions are supported within compatible groups.
- REQ-4.1-03: calculations and assignments require matching units and precision.
- REQ-4.2-01: numeric valid values require units.
- REQ-4.2-02: assessment ranges require units.
- REQ-4.2-03: numeric definitions require units.
- REQ-4.2-04: ask requires a question property on the value.
- REQ-4.2-05: unit requirement errors report line numbers.
- REQ-4.2-06: numbers and enumerations must define valid values.
- REQ-4.2-07: valid value ranges must not overlap.
- REQ-4.3-01: values cannot be used before assignment.
- REQ-4.3-02: calculation properties do not seed values.
- REQ-4.3-03: statistical functions do not require local initialization.
- REQ-4.3-04: listen for and context data initialize values.
- REQ-4.3-05: meaning-of expressions require an askable value when not initialized.
- REQ-4.4-01: meaning ranges must cover valid values (integer gaps).
- REQ-4.4-02: meaning ranges must cover valid values (float gaps).
- REQ-4.4-03: disjoint valid ranges are allowed when fully covered.
- REQ-4.4-04: overlapping numeric assessment ranges are invalid.
- REQ-4.4-05: duplicate enumeration cases are invalid.
- REQ-4.4-06: gap detection reports missing integer spans.
- REQ-4.4-07: gap detection reports missing float spans.
- REQ-4.4-08: coverage gaps respect precision for float and integer ranges.
- REQ-4.4-09: overlapping ranges are rejected.
- REQ-4.4-10: missing coverage yields a validation error.
- REQ-4.4-11: trend assessments require full coverage.
- REQ-4.4-12: numeric valid value ranges use consistent precision across bounds and intervals.
- REQ-4.4-13: valid meanings must be fully used across meaning assessments.
- REQ-4.4-14: meaning labels must be drawn from declared valid meanings.
- REQ-4.5-01: interval math supports range compliance checks.
- REQ-4.5-02: assignment range compliance fails when out of bounds.
- REQ-4.6-01: timeframe calculations require Not enough data handling.
- REQ-4.6-02: Not enough data handling satisfies sufficiency.
- REQ-4.6-03: runtime evaluation returns NotEnoughData when history is insufficient.
- REQ-4.6-04: Not enough data is only allowed for statistical assessments.
- REQ-4.7-01: date/time valid value ranges evaluate using date/time and time-of-day semantics.
- REQ-4.7-02: date diff expressions evaluate to quantities in requested units.
- REQ-5-01: runtime executes assignments and actions in order.
- REQ-5-02: reuse timeframes prevent re-asking within the validity window.
- REQ-5-03: runtime emits a warning when a message action executes without a message callback.
- REQ-5-04: simulation mode executes plans at accelerated speed without real-time delays.
- REQ-5-05: when a periodic trigger includes an `at` time, the first execution is scheduled at that time on the first eligible day, and subsequent executions recur at that same time each interval.
- REQ-5.1-01: full-plan validation passes for a complete plan.
- REQ-5.2-01: numeric answers must respect the decimal precision implied by valid values.
- REQ-5.3-01: meaning-of expressions evaluate using meaning assessments.
- REQ-5.3-02: meaning evaluation returns Missing for unknown askable values.
- REQ-5.3-03: meaning evaluation supports nested assessments.

---

## Revision History

| Rev  | Date       | Author         | Description                                              |
|------|------------|----------------|----------------------------------------------------------|
| 1.0  | 2026-03-20 | (auto-adopted) | Initial adoption of Language Specification v1.0 as SYRS-01. V-Model preamble and traceability matrix added. |
| 1.1  | 2026-03-20 | (auto-adopted) | Added REQ-5-04 (simulation mode). |
| 1.2  | 2026-03-23 | V-Model | Added REQ-3.8-05 (at time clause), REQ-3.8-06 (period repetition), REQ-5-05 (time-pinned scheduling). Updated EBNF grammar. |
| 1.3  | 2026-03-23 | V-Model | Added REQ-3.7-10 (`after plan:` block). Updated EBNF: `plan_block` gains `after_plan_block` alternative. |
| 1.4  | 2026-03-23 | V-Model | Added REQ-3.8-07 (bare unit and ordinal trigger syntax). Updated EBNF: added `ordinal` and `bare_unit` rules, updated `event_trigger`. |
