# Hippocrates Language Model Context Protocol (Version 1.0)

## 1. Introduction

This document formalizes the **Hippocrates Language**, a domain-specific language (DSL) designed for defining medical care plans, protocols, and digital health interventions. The language emphasizes natural language readability, strict typing of physical units, and robust handling of temporal events.

## 2. Language Principles

* **Natural Language Syntax**: Statements mimic English sentences to ensure readability by medical professionals.
* **Type Safety & Units**: All numeric literals must be quantities with units (built-in or custom), e.g., `10 mg` or `7 days`.
* **Contextual Execution**: Scripts execute within a specific context (Patient, Timeframe).
* **Event-Driven**: The core runtime is an event loop reacting to time, value changes, and external triggers.
* **Completeness**: A plan describes a self-contained logic for a single subject (the Patient).
* **Angle-Bracket Identifiers**: All identifiers are written as `<...>`.
* **Indented Blocks**: Any `:` that opens a block requires a newline followed by an indented block.
* **No Comparison Operators**: Use ranges (`min ... max`) instead of `<`, `>`, `<=`, `>=`.

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
```

All identifiers are angle-bracketed. When a rule introduces an indented block (`:` followed by `indent`/`dedent`), a newline is required. Inline `:` forms are only allowed where explicitly shown (e.g., `documentation` strings).

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

All numeric literals in user scripts must be expressed as quantities with units; unitless numbers are invalid.
Built-in units are reserved and cannot be redefined or aliased in unit definitions.

Precision rule: integer ranges (e.g., `0 <points> ... 10 <points>`) use step size 1; decimal ranges (e.g., `0.0 mg ... 10.0 mg`) use the smallest declared decimal precision (step size `10^-precision`).

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

### 3.4. Values

```ebnf
value_definition =
    identifier, " is ", value_type,
    [ ":", newline, indent, { value_property }, dedent ],
    [ "." ];

value_type =
    "a number" | "an enumeration" | "a string" | "a time indication" |
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

meaning_prop = "meaning:", newline, indent, { assessment_case }, dedent;

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
timeframe_line = range_selector, { ";", range_selector }, newline;

unit_ref_prop = "unit", (" is " | ":"), unit;

generic_property = identifier, flexible_property_content;
flexible_property_content =
    ":", newline, indent, property_content, dedent |
    ":", property_line;
property_content = { character };
property_line = { character - newline };
```

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
    during_plan_block |
    trigger_block |
    event_block;

during_plan_block = "during plan:", newline, indent, { statement }, dedent;

trigger_block = event_trigger, ":", newline, indent, { statement }, dedent;

event_block = identifier, " with ", event_trigger, ":", newline, indent, { statement }, dedent;
```

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
meaning_assignment = "meaning of value = ", expression, [ "." ];

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

constraint = expression, constraint_operator, range_selector, [ "!" | "?" | "." ];
constraint_operator = "is" | "during" | "after";

block = { statement };

context_block = "context", [ " for analysis" ], ":", newline, indent, { context_item | statement }, dedent;

timeframe_block =
    "timeframe", [ " for analysis" ],
    [ constraint_operator, range_selector, { constraint_operator, range_selector } ],
    ":", newline, indent, { statement }, dedent;
```

### 3.7. Actions and Questions

```ebnf
action =
    show_message |
    ask_question |
    listen_for |
    send_info |
    question_modifier |
    message_expiration |
    start_period |
    simple_command;

show_message =
    "show message", [ " to ", ( "patient" | "physician" | identifier ) ],
    flexible_message_content, [ flexible_block ], [ "." ];

flexible_message_content = expression, { newline, expression };

ask_question =
    "ask", [ " for" | " patient" | " physician" ], ( string_literal | identifier ),
    [ flexible_block ], [ "." ];

flexible_block = ":", newline, indent, { statement }, dedent;

listen_for = "listen for ", identifier, ":", newline, indent, { statement }, dedent;

send_info = "send information ", string_literal, { expression | newline }, [ "." ];

start_period = "start ", identifier, [ "." ];

simple_command = identifier, [ "." ];

message_expiration = "message expires after ", range_selector, [ "." ];

question_modifier =
    "question expires after ", period_expr, [ flexible_block ] |
    validate_modifier |
    "type of question is ", string_literal, [ "." ] |
    "style of question is ", identifier, [ "." ] |
    "question style is visual analogue scale:", newline, indent, vas_block, dedent;

validate_modifier =
    "validate answer ", validation_mode, [ " within ", quantity ], [ "." ], [ flexible_block ];
validation_mode = "once" | "twice";

vas_block = { best_value_def | best_label_def | worst_value_def | worst_label_def };

best_value_def = "best value is ", quantity, [ "." ], newline;
best_label_def = "text for best value is ", string_literal, [ "." ], newline;
worst_value_def = "worst value is ", quantity, [ "." ], newline;
worst_label_def = "text for worst value is ", string_literal, [ "." ], newline;
```

Question waits do not block subsequent loop triggers. If the next scheduled trigger occurs before an answer arrives, the engine resumes the loop with the question still pending (the block may re-ask or continue). If a question expires without an answer, an optional `question expires after` block runs at expiration time and can send a reminder or log a message.

### 3.8. Events and Timing

```ebnf
event_trigger =
    "change of ", identifier |
    "begin of ", identifier |
    "every ", quantity, [ identifier ], [ " for ", quantity ] |
    "every ", ( identifier | weekday ), [ " after ", identifier ], [ " for ", quantity ];

period_expr = quantity | "until ", event_trigger;
```

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

### 3.11. Data Contexts

```ebnf
context_definition = "context:", newline, indent, { context_item }, dedent;

context_item =
    "timeframe:", range_selector |
    "data:", identifier |
    "value filter:", assessment_case;
```

### 3.12. Expressions and Statistical Analysis

```ebnf
expression = term, { infix_op, term };

term =
    quantity, relative_time_modifier |
    statistical_func |
    quantity |
    time_indication |
    string_literal |
    identifier |
    "(", expression, ")";

statistical_func =
    ("count of" | "min of" | "max of" | "trend of"), identifier, [ " is ", term ] |
    "average of", identifier, (" over " | " for "), quantity;

infix_op = "+" | "-" | "*" | "/";
relative_time_modifier = "ago" | "from now";
```

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

The runtime ensures that values compared or assigned have compatible units (belong to the same group).

#### Unit Normalization

For custom units (e.g., `points`, `tablets`), pluralization and abbreviations must be defined explicitly. Without a definition, `10 <point>` and `10 <points>` are treated as different units.

```hippocrates
<point> is a unit:
    plural is <points>
    abbreviation is "pts"
```

### 4.2. Required Properties

* **Numbers and Enumerations**: `valid values` must be defined.
* **Numbers**: A unit must be defined (via `unit is ...` or by using quantities in `valid values`).
* **Asking**: `ask` is only valid when a value has a `question` property.

### 4.3. Data Flow and Validity

* A value cannot be used before it has valid content.
* Values gain valid content by being assigned, asked, or provided by `listen for` or `context data:`.
* Calculation properties describe how a value is derived but do not implicitly seed it; plans must assign or ask before use.
* Statistical functions read history and do not require local initialization of the referenced value.

### 4.4. Assessment Coverage

* `assess` blocks, `meaning` cases, and assessments over statistical results must fully cover the valid range of the target/output.
* For enumerations, all valid values must be covered.
* For `trend of <value>`, all cases (`"increase"`, `"decrease"`, `"stable"`) must be covered.

### 4.5. Range Compliance (Pre-Run Validation)

Before execution, the runtime validates that calculated and assigned values remain within their declared ranges. If the computed range can exceed the valid values, validation fails.

### 4.6. Data Sufficiency

Calculations involving history use `Not enough data` when the available history is shorter than the requested timeframe. This is handled explicitly in assessments.

## 5. Execution Model

The Hippocrates Runtime functions as a **State Machine**.

1. **Load**: Parse script, build internal dependencies graph (DAG).
2. **Init**: Initialize all values to `unknown` or default; restore state from persistence.
3. **Loop**:
    * Check **Timers**: Are there any temporal events (`every 1 day`, `every Monday`)? -> Trigger Event.
    * Check **Inputs**: Did an external API update a value? -> Trigger `change of` Event.
    * Evaluate **Rules**: If an Event triggered, execute the associated `block`.
    * **Side Effects**: Execute `show`, `ask`, or `send information` commands via API callbacks.

### 5.1 Validation Logic

Before execution, the runtime validates that:

1. All `assess` blocks and `meaning` cases cover the complete valid range of the target value.
2. No values are used before they are initialized or asked.
3. All referenced variables and units are compatible.

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
            <inhaler used in past 5 days> = count of <inhaler used> is "Yes".

<inhaler used in past 5 days on time> is a number:
    valid values:
        0 <doses> ... 1000 <doses>
    calculation:
        timeframe for analysis is between 5 days ago ... now during <best inhalation period>:
            <inhaler used in past 5 days on time> = count of <inhaler used> is "Yes".
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

during plan:
    assess <weekly average>:
        Not enough data:
            show message "Please continue tracking pain for a full week.".
        0 <points> ... 5 <points>:
            show message "Your pain levels are within range this week.".
        6 <points> ... 10 <points>:
            show message "Your pain levels are high this week.".
```

### 6.4. Message Expiration

```hippocrates
show message to <patient> "Take your medication now":
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
                show message "We still need your answer for today's pain score.".
```

### 6.6. Validity Timeframe (Reuse)

```hippocrates
<body temperature> is a number:
    valid values:
        35.0 °C ... 42.0 °C
    reuse:
        reuse period of value is 1 hour.

during plan:
    ask <body temperature>.
```
