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
string_literal = '"', { character - ( "<" | ">" ) }, '"';
identifier = ( "<", { character - ">" }, ">" ) | ( word, { " ", word } );

(* Basic Types *)
percentage = ( "0" | "100" | digit, [ digit ] ), "%";
time_unit = "year" | "month" | "week" | "day" | "hour" | "minute" | "second";
period_of_time = 
    integer, " ", time_unit, [ "s" ] |
    "until ", event_trigger; (* e.g., 2 weeks, until next visit *)

(* Units *)
(* Units *)
temperature_unit = "°F" | "°C";
weight_unit = "mg" | "g" | "kg" | "lb" | "oz";
length_unit = "m" | "cm" | "mm" | "km" | "inch" | "foot" | "mile";
volume_unit = "l" | "ml" | "fl oz" | "gal";
time_unit = "year" | "month" | "week" | "day" | "hour" | "minute" | "second";

(* Custom Units *)
(* Custom units must now be explicitly defined to support pluralization and abbreviations. *)
(* If not defined, "drop" and "drops" are treated as distinct, unrelated units. *)
unit = temperature_unit | weight_unit | length_unit | volume_unit | time_unit | identifier; 

(* Unit Definition *)
unit_definition = 
    identifier, " is a unit:", newline,
        indent, { unit_property }, dedent;
        
unit_property = 
    "plural is ", string_literal |
    "singular is ", string_literal |
    "abbreviation is ", string_literal;
    
number = integer | float;
(* Precision Rule: Ranges like 0 ... 10 imply integer steps; 0.0 ... 10.0 imply float steps *)
quantity = number, [ " " ], unit;
```

### 3.2. Program Structure

```ebnf
hippocrates_file = 
    [ intended_use_chapter ],
    [ library_reference_chapter ],
    [ settings_chapter ],
    { definition_chapter },
    plan_chapter;

plan_chapter = 
    identifier, " is a plan:", newline,
        indent, { plan_block }, dedent;

plan_block = 
    "during plan:", newline, indent, { statement }, dedent |
    trigger_block | 
    identifier, " with ", event_trigger, ":", newline, indent, { statement }, dedent;


definition_chapter = 
    addressee_chapter |
    value_definition_chapter |
    period_definition_chapter |
    event_definition_chapter;

period_definition_chapter = 
    identifier, " is a period:", newline,
        indent, { period_property }, dedent;

period_property = 
    "timeframe: ", { range_selector } |
    "customization:", newline, indent, block, dedent;

```

### 3.3. Values

Values are the core state containers.

```ebnf
value_definition = 
    identifier, " is ", value_type, ":", newline,
        indent, { value_property }, dedent;

value_type = "a number" | "an enumeration" | "a time indication";

value_property = 
    intended_use_prop |
    inheritance_prop |
    valid_units_prop |
    valid_values_prop |
    meaning_prop |
    question_prop |
    calculation_prop |
    reuse_prop;

meaning_prop = "meaning:", newline, indent, { assessment_case }, dedent;

inheritance_prop = "definition is the same as for ", identifier, [ " except:", newline, indent, { value_property }, dedent ];

question_prop = "question:", newline, indent, ask_question_block, dedent;

ask_question_block = 
    ask_question_statement, 
    [ "question expires after ", period_of_time, [ ":", newline, indent, block, dedent ] ],
    [ "validate answer ", ("once" | "twice"), [ " within ", period_of_time ], [ ":", newline, indent, block, dedent ] ],
    [ "type of question is ", string_literal, "." ],
    [ "style of question is ", identifier, "." ],
    [ "question style is visual analogue scale:", newline, indent, vas_block, dedent ];

vas_block = 
    ( "best value is ", ( number | quantity ), ":", newline, indent, "text for best value is ", string_literal, ".", dedent ),
    ( "worst value is ", ( number | quantity ), ":", newline, indent, "text for worst value is ", string_literal, ".", dedent );


question_style = "Likert" | "visual analogue scale" | "selection" | "text" | "number" | "date";

valid_values_prop = "valid values: ", { range_selector };

calculation_prop = "calculation:", newline, indent, ( calculation_statement | timeframe_block ), dedent;

timeframe_block = 
    "timeframe for analysis is ", range_selector, ":", newline, 
    indent, { calculation_statement }, dedent;

reuse_prop = "reuse:", newline, indent, "reuse period of value is ", quantity, ".", dedent;

(* Example: 
<body weight> is a number:
    valid values: 0 kg ... 200 kg

*)
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

ask_question_statement = 
    "ask", [ " for ", ("patient" | "physician") ], ( string_literal | identifier ), [ ".", block ];

start_period_statement = 
    "start ", identifier, ".";

(* Conditionals *)
conditional_statement = 
    "assess ", assessment_target, ":", newline,
        indent, { assessment_case },
    dedent;

*Constraint: The union of all `assessment_case` conditions MUST cover the entire valid range of the `assessment_target`. implicit default cases are not allowed.*

assessment_target = identifier | "confidence of ", identifier;

assessment_case = range_selector, ":", newline, indent, block, dedent;

(* Ranges *)
range_selector = 
    "between ", expression, " ... ", expression, |
    expression | (* Equality check *)
    "Not enough data";

```

### 3.5. Events and Timing

```ebnf
event_listener = "catch ", event_trigger, ":", newline, indent, block, dedent;

event_trigger = 
    "change of ", identifier |
    "every ", period_of_time |
    "at ", time_of_day;
```

### 3.6. Communication & Actors

Hippocrates manages communication through `addressees`.

```ebnf
addressee_chapter = 
    identifier, " is an addressee:", newline,
        indent, { addressee_property }, dedent |
    identifier, " is an addressee group:", newline,
        indent, { addressee_group_property }, dedent;

addressee_property = 
    "contact information:", newline, indent, { contact_detail }, dedent |
    "after consent has been rejected:", newline, indent, block, dedent;

contact_detail = 
    "email is ", string_literal |
    "phone is ", string_literal |
    "hippocrates id is ", string_literal;

addressee_group_property = 
    "grouped addressees are ", identifier, { "; ", identifier } |
    "order of contacting:", newline, indent, contact_logic, dedent;

contact_logic = 
    "contact all addressees in parallel" |
    "sequence of contacting is ", identifier, { "; ", identifier };
```

#### Messaging Actions

The `action_statement` is expanded to support rich messaging.

```ebnf
action_statement = 
    ("show" | "send"), " message to ", identifier, " ", string_literal, [ ":", newline, indent, message_options, dedent ];

message_options = 
    "message expires after ", period_of_time |
    "after delivery has failed:", newline, indent, block, dedent |
    "after delivery has succeeded:", newline, indent, block, dedent;
```

### 3.7. Medication

Drugs are a first-class entity in Hippocrates.

```ebnf
drug_definition = 
    identifier, " is a drug:", newline,
        indent, 
        [ ingredients_block ],
        [ dosage_safety_block ],
        [ administration_block ],
        [ interactions_block ],
        dedent;

ingredients_block = "ingredients:", newline, indent, { ingredient }, dedent;
ingredient = identifier, " ", float, " ", weight_unit;

dosage_safety_block = "dosage safety:", newline, indent, { dosage_rule }, dedent;
dosage_rule = 
    "maximum single dose = ", expression |
    "maximum daily dose = ", expression |
    "minimum time between doses = ", period_of_time;

administration_block = "administration:", newline, indent, { admin_rule }, dedent;
admin_rule = 
    "form of administration is ", identifier |
    identifier, " ", period_of_time, " after ", identifier; (* Schedule *)

interactions_block = "interactions:", newline, indent, { interaction_rule }, dedent;
interaction_rule = "assess interaction with ", identifier, ":", newline, indent, block, dedent;
```

#### Example (Medication)

```hippocrates
<Paracetamol> is a drug:
    ingredients:
        acetaminophen 500 mg
    dosage safety:
        maximum single dose = 1000 mg
        maximum daily dose = 4000 mg
        minimum time between doses = 4 hours
    administration:
        form of administration is tablet
```

### 3.8. Data Contexts

Contexts are used to filter data for analysis or decision making in `diagnosis`.

```ebnf
context_definition = 
    "context:", newline, indent, { context_item }, dedent;

context_item = 
    "timeframe: ", range_selector |
    "data: ", identifier |
    "value filter: ", assessment_case;
```

#### Example (Data Contexts)

```hippocrates
diagnosis:
    context:
        timeframe: 2 weeks ago ... now
        data: blood pressure
    assess using context:
        ...
```

### 3.10. Statistical Analysis

Statistical functions are first-class citizens in `diagnosis` blocks.

```ebnf
statistical_function = 
    "count of ", identifier, [ " is ", expression ] |
    "average of ", identifier, [ " over ", period_of_time ] |
    "min of ", identifier |
    "max of ", identifier |
    "trend of ", identifier;

```

### 3.11. Event Listening

Background listening for value changes (e.g., from devices).

```ebnf
listen_statement = 
    "listen for ", identifier, ":", newline,
        indent, block, dedent;
```

## 4. Semantics and Type System

### 4.1. Core Unit Groups and Conversion

The runtime natively understands and automatically converts between units within the following groups. No user definition is required.

* **Mass**: mg, g, kg, lb, oz
* **Length**: m, cm, mm, km, inch, foot, mile
* **Volume**: ml, l, fl oz, gal
* **Time**: sec, min, hour, day, week, month, year
* **Temperature**: °C, °F

The runtime MUST ensure that values compared or assigned have compatible units (belong to the same group). Conversions (e.g., `lb` to `kg`) are performed automatically.

#### Unit Normalization

For custom units (e.g., `points`, `tablets`), the runtime **no longer** automatically normalizes plural forms. You MUST explicitly define the relationship:

```hippocrates
point is a unit:
    plural is "points".
```

* Without this definition, `10 points` and `10 point` are considered effectively different units (though the runtime logic might treat them as distinct unit strings).
* With the definition, `10 points` is canonicalized to `10 point` (using the definition name as canonical).

### 4.2. Confidence and History

Every value in Hippocrates has meta-properties managed by the runtime:

* `value.timestamp`: When was this value last updated.
* `value.confidence`: A percentage (0-100%) indicating certainty.
  * Calculated values inherit the *lowest* confidence of their inputs.
  * Values are considered "valid" for reuse only if their age is within the defined `reuse` period.
  * **Rule**: When an action requires a value (e.g. `ask`), the system checks if a valid historical value exists.
    * If `age < reuse_period`: The question is skipped, and the historical value is used.
    * If `age >= reuse_period` (or value missing): The system prompts the user for a new value.

### 4.3. Context Resolution

Variables (Values) are resolved in the following order:

1. **Local Scope**: Variables defined within the current block (e.g., iterator variables).
2. **Global Scope**: Values defined in the `value definition` chapters.
3. **Library Scope**: Values imported from linked libraries.

### 4.4. Data Sufficiency

Calculations involving timeframes (e.g., `count of ... in past 5 days`) enforce strict data sufficiency rules.

* If the system has been running for less time than the requested timeframe (e.g., running for 2 days, requested 5 days), the result is `Not Enough Data`.
* This ensures that calculations do not return partial or misleading results (e.g., returning 0 incidents simply because history is empty).
* Plans can explicitly handle this state:

```hippocrates
assess <incident count>:
    Not enough data:
        show message "Collecting more data...".
    > 5:
        show message "High incidents".
```

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

### 6.1 Validation Logic

Before execution, the runtime validates that:

1. All `assess` blocks cover the complete valid range of the target value.
2. No ambiguous or overlapping conditions exist.
3. All referenced variables and units are compatible.

## 7. Examples by Feature

### 7.1. Visual Analogue Scale (VAS)

```hippocrates
<pain level> is a number:
    valid values: 0 ... 10
    question:
        ask "How severe is your pain?":
            question style is visual analogue scale:
                best value is 0:
                    text for best value is "No pain".
                worst value is 10:
                    text for worst value is "Worst pain imaginable".
```

### 7.2. Filtered Calculations with Timeframes

```hippocrates
<inhaler used in past 5 days> is a number:
    calculation:
        timeframe for analysis is between 5 days ago ... now:
            value = count of inhaler used is yes.
```

### 7.3. Handling Insufficient Data

```hippocrates
<weekly average> is a number:
    calculation:
        timeframe for analysis is 7 days ago ... now:
            value = average of <daily pain> over 7 days.

during plan:
    assess <weekly average>:
        Not enough data:
            show message "Please continue tracking pain for a full week.".
        > 5:
            show message "Your pain levels are high this week.".
```

### 7.3. Message Expiration

```hippocrates
show message to patient "Take your medication now":
    message expires after 15 minutes.

### 7.4. Validity Timeframe (Reuse)

```hippocrates
<Body Temperature> is a number:
    reuse:
        reuse period of value is 1 hour.

during plan:
    ask <Body Temperature>.
    // If run again within 1 hour, this question is skipped.
```

```
