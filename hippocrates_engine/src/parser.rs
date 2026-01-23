use crate::ast::*;
use crate::domain::*;
use chrono::{NaiveDate, NaiveDateTime};
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HippocratesParser;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parsing error: {0}")]
    PestError(#[from] pest::error::Error<Rule>),
    #[error("Unknown unit: {0}")]
    UnknownUnit(String),
    #[error("Unknown rule: {0}")]
    UnknownRule(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub fn parse_plan(input: &str) -> Result<Plan, EngineError> {
    let processed = preprocess_indentation(input);

    
    let pairs = match HippocratesParser::parse(Rule::file, &processed) {
        Ok(p) => p,
        Err(e) => {
            return Err(to_engine_error(ParseError::PestError(e)));
        }
    };

    let mut definitions = Vec::new();

    let root = pairs.into_iter().next().unwrap();
    for pair in root.into_inner() {
        match pair.as_rule() {
            Rule::definition => {
                let inner = pair.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::value_definition => {
                        definitions.push(Definition::Value(parse_value_def(inner).map_err(to_engine_error)?))
                    }
                    Rule::period_definition => {
                        definitions.push(Definition::Period(parse_period_def(inner).map_err(to_engine_error)?))
                    }
                    Rule::plan_definition => {
                        definitions.push(Definition::Plan(parse_plan_def(inner).map_err(to_engine_error)?))
                    }
                    Rule::drug_definition => {
                        definitions.push(Definition::Drug(parse_drug_def(inner).map_err(to_engine_error)?))
                    }
                    Rule::addressee_definition => {
                        definitions.push(Definition::Addressee(parse_addressee_def(inner).map_err(to_engine_error)?))
                    }
                    Rule::context_definition => {
                        definitions.push(Definition::Context(parse_context_def(inner).map_err(to_engine_error)?))
                    }
                    Rule::unit_definition => {
                        definitions.push(Definition::Unit(parse_unit_def(inner).map_err(to_engine_error)?))
                    }
                    _ => (),
                }
            }
            Rule::EOI => (),
             _ => (),
        }
    }

    let plan = Plan { definitions };
    
    // Validation is now the responsibility of the caller (to support multi-error)
    
    Ok(plan)
}

fn to_engine_error(e: ParseError) -> EngineError {
    match e {
        ParseError::PestError(pe) => {
            let line_col = match pe.line_col {
                pest::error::LineColLocation::Pos((line, col)) => (line, col),
                pest::error::LineColLocation::Span((line, col), _) => (line, col),
            };
            
            // Custom simplified error message
            let message = match pe.variant {
                pest::error::ErrorVariant::ParsingError { positives, negatives } => {
                     let expected: Vec<String> = positives.iter().map(|r| format!("{:?}", r)).collect();
                     let mut msg = String::new();
                     if !expected.is_empty() {
                         msg.push_str("Expected ");
                         // simple join
                         msg.push_str(&expected.join(", "));
                     }
                     if !negatives.is_empty() {
                         if !msg.is_empty() { msg.push_str("; "); }
                         msg.push_str("Unexpected ");
                         let unexpected: Vec<String> = negatives.iter().map(|r| format!("{:?}", r)).collect();
                         msg.push_str(&unexpected.join(", "));
                     }
                     if msg.is_empty() {
                         "Parsing error".to_string()
                     } else {
                         msg
                     }
                }
                pest::error::ErrorVariant::CustomError { message } => message,
            };

            EngineError {
                message,
                line: line_col.0,
                column: line_col.1,
            }
        },
        ParseError::ValidationError(msg) => EngineError {
            message: msg,
            line: 0,
            column: 0,
        },
        ParseError::UnknownRule(msg) => EngineError {
            message: format!("Unknown rule: {}", msg),
            line: 0,
            column: 0,
        },
        ParseError::UnknownUnit(msg) => EngineError {
            message: format!("Unknown unit: {}", msg),
            line: 0,
            column: 0,
        }
    }
}

/// Converts indentation to explicit tokens '《' (INDENT) and '》' (DEDENT)
/// This allows Pest to parse significant whitespace.
pub fn preprocess_indentation(input: &str) -> String {
    let mut output = String::new();
    let mut indent_stack = vec![0];

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            output.push('\n');
            continue;
        }

        // Calculate indentation (assuming spaces)
        let indent = line.chars().take_while(|c| *c == ' ').count();
        let current = indent_stack.last().copied().unwrap_or(0);

        if indent > current {
            indent_stack.push(indent);
            output.push('《');
            // output.push('\n'); // REMOVED to preserve line count
        }

        while indent < *indent_stack.last().unwrap() {
            indent_stack.pop();
            output.push('》');
            // output.push('\n'); // REMOVED
        }

        output.push_str(trimmed);
        output.push('\n');
    }

    // Dedent remaining at EOF
    while indent_stack.len() > 1 {
        indent_stack.pop();
        output.push('》');
        // output.push('\n'); // REMOVED
    }

    output
}

// -----------------------------------------------------------------------------
// Definition Parsers
// -----------------------------------------------------------------------------

fn parse_value_def(pair: pest::iterators::Pair<Rule>) -> Result<ValueDef, ParseError> {
    let mut inner = pair.into_inner();
    let name_pair = inner.next().unwrap();
    let name = parse_identifier_str(name_pair);

    let type_pair = inner.next().unwrap();
    let value_type = match type_pair.as_str() {
        "a number" => ValueType::Number,
        "an enumeration" => ValueType::Enumeration,
        "a string" => ValueType::String,
        "a time indication" => ValueType::TimeIndication,
        "a date/time" => ValueType::DateTime,
        "a period" => ValueType::Period,
        "a plan" => ValueType::Plan,
        "a drug" => ValueType::Drug,
        "an addressee" => ValueType::Addressee,
        "an addressee group" => ValueType::AddresseeGroup,
        _ => ValueType::Number,
    };

    let mut properties = Vec::new();
    for prop in inner {
        properties.push(parse_value_property(prop)?);
    }

    Ok(ValueDef {
        name,
        value_type,
        properties,
    })
}

fn parse_period_def(pair: pest::iterators::Pair<Rule>) -> Result<PeriodDef, ParseError> {
    let line = pair.as_span().start_pos().line_col().0;
    let mut inner = pair.into_inner();
    let name = parse_identifier_str(inner.next().unwrap());

    let mut timeframes = Vec::new();

    for prop in inner {
        match prop.as_rule() {
            Rule::period_property => {
                let s = prop.as_str().trim();
                if s.starts_with("timeframe") {
                    let p_inner = prop.into_inner();
                    // Depending on grammar structure, we might need to skip literals or find timeframe_line directly
                    // period_property = { "timeframe" ... timeframe_line+ ... }
                    for child in p_inner {
                        if child.as_rule() == Rule::timeframe_line {
                             let mut selectors = Vec::new();
                             for sel in child.into_inner() {
                                 if sel.as_rule() == Rule::timeframe_selector {
                                     selectors.push(parse_range_selector(sel)?);
                                 }
                             }
                             if !selectors.is_empty() {
                                 timeframes.push(selectors);
                             }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(PeriodDef {
        name,
        timeframes,
        line,
    })
}

fn parse_plan_def(pair: pest::iterators::Pair<Rule>) -> Result<PlanDef, ParseError> {
    let mut inner = pair.into_inner();
    let name = parse_identifier_str(inner.next().unwrap());

    let mut blocks = Vec::new();
    for block_pair in inner {
        blocks.push(parse_plan_block(block_pair)?);
    }

    Ok(PlanDef { name, blocks })
}

fn parse_plan_block(pair: pest::iterators::Pair<Rule>) -> Result<PlanBlock, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::during_plan_block => {
            // flexible_block is silent, so we get block_body directly
            let block_body = inner.into_inner().next().unwrap();
            let stmts = parse_block(block_body)?;
            Ok(PlanBlock::DuringPlan(stmts))
        }
        Rule::event_block => {
            let mut e_inner = inner.into_inner();
            let name = parse_identifier_str(e_inner.next().unwrap());
            let trigger = parse_event_trigger(e_inner.next().unwrap())?;
            let block_body = e_inner.next().unwrap();
            let stmts = parse_block(block_body)?;
            Ok(PlanBlock::Event(EventBlock {
                name,
                trigger,
                statements: stmts,
            }))
        }
        Rule::trigger_block => {
            let mut t_inner = inner.into_inner();
            let trigger = parse_event_trigger(t_inner.next().unwrap())?;
            let block_body = t_inner.next().unwrap();
            let stmts = parse_block(block_body)?;
            Ok(PlanBlock::Trigger(TriggerBlock {
                trigger,
                statements: stmts,
            }))
        }
        _ => Err(ParseError::UnknownRule(format!("{:?}", inner.as_rule()))),
    }
}

fn parse_drug_def(pair: pest::iterators::Pair<Rule>) -> Result<DrugDef, ParseError> {
    let mut inner = pair.into_inner();
    let name = parse_identifier_str(inner.next().unwrap());

    // inner (after name) contains ingredients_block, dosage_block, etc.
    // These are top level blocks in grammar for drug_definition
    // drug_definition = { identifier ~ "is a drug" ~ ":" ... (ingredients_block | ...)* ... }
    // These blocks are NOT wrapped in value_property.

    let mut properties = Vec::new();
    for p in inner {
        match p.as_rule() {
            Rule::ingredients_block => {
                let mut ingredients = Vec::new();
                for i_pair in p.into_inner() {
                    // ingredient rule
                    let mut i_inner = i_pair.into_inner();
                    let i_name = parse_identifier_str(i_inner.next().unwrap());
                    let qty_pair = i_inner.next().unwrap();
                    let (i_amt, i_unit) = parse_quantity_pair(qty_pair)?;
                    ingredients.push(Ingredient {
                        name: i_name,
                        amount: i_amt,
                        unit: i_unit,
                    });
                }
                properties.push(Property::Ingredients(ingredients));
            }
            Rule::dosage_block => {
                let mut rules = Vec::new();
                for r_pair in p.into_inner() {
                    let rule_str = r_pair.as_str();
                    let mut r_inner = r_pair.into_inner();
                    if let Some(expr_pair) = r_inner.next() {
                        let expr = parse_expression(expr_pair)?;
                        if rule_str.starts_with("maximum single") {
                            rules.push(DosageRule::MaxSingle(expr));
                        } else if rule_str.starts_with("maximum daily") {
                            rules.push(DosageRule::MaxDaily(expr));
                        } else {
                            rules.push(DosageRule::MinTimeBetween(expr));
                        }
                    }
                }
                properties.push(Property::DosageSafety(rules));
            }
            Rule::admin_block => {
                let mut rules = Vec::new();
                for admin_pair in p.into_inner() {
                    let inner_rule = admin_pair.into_inner().next().unwrap();
                    match inner_rule.as_rule() {
                        Rule::admin_form_rule => {
                            let mut form_inner = inner_rule.into_inner();
                            // "form of..." literal is ignored by pest if atomic? Or matches?
                            // grammar: "form of..." ~ (identifier | string_literal)
                            // Usually string literals in rules are not produced as pairs unless named.
                            // So the first pair inside admin_form_rule should be the value.
                            // Check if "form of..." is produced. It is atomic string string literal in rule.
                            // In pest, if rule is compound {}, literals are silent unless rule is explicit?
                            // "foo" produces no token.
                            // So next() should be the value pair.
                            let value_pair = form_inner.next().unwrap();
                            let ident = parse_identifier_str(value_pair);
                            rules.push(AdminRule::Form(ident));
                        }
                        Rule::admin_schedule_rule => {
                            let mut sched_inner = inner_rule.into_inner();
                            let drug1 = parse_identifier_str(sched_inner.next().unwrap());
                            let qty_expr = parse_expression(sched_inner.next().unwrap())?;
                            let drug2 = parse_identifier_str(sched_inner.next().unwrap());
                            rules.push(AdminRule::Schedule(drug1, qty_expr, drug2));
                        }
                        _ => {}
                    }
                }
                properties.push(Property::Administration(rules));
            }
            Rule::interaction_block => {
                let mut rules = Vec::new();
                for i_pair in p.into_inner() {
                    let mut i_inner = i_pair.into_inner();
                    let drug_name = parse_identifier_str(i_inner.next().unwrap());
                    let block = parse_block(i_inner.next().unwrap())?;
                    rules.push(InteractionRule {
                        drug: drug_name,
                        block,
                    });
                }
                properties.push(Property::Interactions(rules));
            }
            _ => {}
        }
    }

    Ok(DrugDef { name, properties })
}

fn parse_addressee_def(pair: pest::iterators::Pair<Rule>) -> Result<AddresseeDef, ParseError> {
    let full_text = pair.as_str().to_string(); // Borrow text before move
    let mut inner = pair.into_inner();
    let name = parse_identifier_str(inner.next().unwrap());

    // "Doctor is an addressee group:"
    let is_group = full_text.contains("addressee group");

    let mut properties = Vec::new();
    for p in inner {
        match p.as_rule() {
            Rule::contact_info_prop => {
                let mut details = Vec::new();
                for c in p.into_inner() {
                    // contact_detail = { type ~ "is" ~ string }
                    let mut c_inner = c.into_inner();
                    let type_str = c_inner.next().unwrap().as_str(); // e.g. "email"
                    let val = parse_string_literal(c_inner.next().unwrap());
                    if type_str == "email" {
                        details.push(ContactDetail::Email(val));
                    } else if type_str == "phone" {
                        details.push(ContactDetail::Phone(val));
                    } else {
                        details.push(ContactDetail::HippocratesId(val));
                    }
                }
                properties.push(Property::ContactInfo(details));
            }
            Rule::grouped_addressees_prop => {
                let mut groups = Vec::new();
                for g in p.into_inner() {
                    groups.push(parse_identifier_str(g));
                }
                properties.push(Property::GroupedAddressees(groups));
            }
            Rule::contact_logic_prop => {
                let l_inner = p.into_inner().next().unwrap();
                // contact_logic = { "parallel" | "sequence" ... }
                if l_inner.as_str().contains("parallel") {
                    properties.push(Property::ContactOrder("Parallel".to_string()));
                } else {
                    properties.push(Property::ContactOrder("Sequence".to_string())); // Simplified
                }
            }
            Rule::after_consent_prop => {
                let blk = parse_block(p.into_inner().next().unwrap())?;
                properties.push(Property::AfterConsentRejected(blk));
            }
            _ => {}
        }
    }

    Ok(AddresseeDef {
        name,
        is_group,
        properties,
    })
}

fn parse_context_def(pair: pest::iterators::Pair<Rule>) -> Result<ContextDef, ParseError> {
    let mut items = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::context_item {
            let s = p.as_str().trim(); // Borrow before move
            let i_inner = p.into_inner();
            // Skip the identifier/string token if it's there?
            // i_inner will contain the sub-rule (range_selector etc)
            // context_item = { ( "timeframe" ... ) ... }
            // If timeframe, inner has "timeframe", ":", range_selector.
            // We need to find the range_selector.
            // We can just iterate until we find match.

            if s.starts_with("timeframe") {
                // Find range_selector
                for child in i_inner {
                    if child.as_rule() == Rule::timeframe_selector {
                        items.push(ContextItem::Timeframe(parse_range_selector(child)?));
                        break;
                    }
                }
            } else if s.starts_with("data") {
                for child in i_inner {
                    if child.as_rule() == Rule::identifier
                        || child.as_rule() == Rule::string_literal
                    {
                        items.push(ContextItem::Data(parse_identifier_str(child)));
                        break;
                    }
                }
            } else if s.starts_with("value filter") {
                for child in i_inner {
                    if child.as_rule() == Rule::assessment_case {
                        let ac_pairs = parse_assessment_case(child)?;
                        if let Some(first) = ac_pairs.into_iter().next() {
                            items.push(ContextItem::ValueFilter(first));
                        }
                        break;
                    }
                }
            }
        }
    }
    Ok(ContextDef { items })
}

fn parse_unit_def(pair: pest::iterators::Pair<Rule>) -> Result<UnitDef, ParseError> {
    let mut inner = pair.into_inner();
    let name = parse_identifier_str(inner.next().unwrap());

    let mut plurals = Vec::new();
    let mut singulars = Vec::new();
    let mut abbreviations = Vec::new();

    for prop in inner {
        let s = prop.as_str().trim().to_string(); // Capture string before move
        let mut p_inner = prop.into_inner();
        // Keyword literals ("plural", "is") are not tokens.
        
        let value_pair = p_inner.next().unwrap(); 
        
        if s.starts_with("plural") {
            let val = parse_identifier_str(value_pair);
            plurals.push(val);
        } else if s.starts_with("singular") {
            let val = parse_string_literal(value_pair);
            singulars.push(val);
        } else if s.starts_with("abbreviation") {
            let val = parse_string_literal(value_pair);
            abbreviations.push(val);
        }
    }

    Ok(UnitDef {
        name,
        plurals,
        singulars,
        abbreviations,
    })
}

// Helper to parse quantity pair (value, unit)
fn parse_quantity_pair(pair: pest::iterators::Pair<Rule>) -> Result<(f64, Unit), ParseError> {
    let mut inner = pair.into_inner();
    let number_pair = inner.next().unwrap();
    let unit_pair = inner.next().unwrap();

    let val = number_pair.as_str().parse::<f64>().map_err(|e| {
        ParseError::PestError(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: e.to_string(),
            },
            number_pair.as_span(),
        ))
    })?;
    let unit = parse_unit(unit_pair)?;
    Ok((val, unit))
}

fn parse_event_trigger(pair: pest::iterators::Pair<Rule>) -> Result<Trigger, ParseError> {
    let s = pair.as_str();
    if s.starts_with("change of") {
        let inner = pair.into_inner();
        for child in inner {
            if child.as_rule() == Rule::identifier {
                return Ok(Trigger::ChangeOf(parse_identifier_str(child)));
            }
            if child.as_rule() == Rule::multi_word_identifier {
                return Ok(Trigger::ChangeOf(parse_multi_word_identifier(child)));
            }
        }
        // Fallback or Error
        return Ok(Trigger::ChangeOf(s.trim_start_matches("change of").trim().to_string()));
    } else if s.starts_with("begin of") {
        let inner = pair.into_inner();
        for child in inner {
            if child.as_rule() == Rule::identifier {
                 return Ok(Trigger::StartOf(parse_identifier_str(child)));
            }
            if child.as_rule() == Rule::multi_word_identifier {
                return Ok(Trigger::StartOf(parse_multi_word_identifier(child)));
            }
        }
        return Ok(Trigger::StartOf(s.trim_start_matches("begin of").trim().to_string())); // Fallback
    } else if s.starts_with("every") {
        let inner = pair.into_inner();
        let mut quantities = Vec::new();
        let mut anchor = None;
        let mut specific_day = None;

        for child in inner {
            match child.as_rule() {
                Rule::quantity => quantities.push(parse_quantity_pair(child)?),
                Rule::identifier => {
                     anchor = Some(parse_identifier_str(child));
                }
                Rule::weekday => {
                     specific_day = Some(child.as_str().to_string());
                }
                _ => {}
            }
        }

        let mut interval_val = 1.0;
        let mut interval_unit = Unit::Second;
        let mut duration = None;

        if let Some(_day) = &specific_day {
             interval_val = 1.0;
             interval_unit = Unit::Week;
        } else if let Some((v, u)) = quantities.get(0) {
             interval_val = *v;
             interval_unit = u.clone();
             if quantities.len() > 1 {
                 duration = Some(quantities[1].clone());
             }
        }
        
        if specific_day.is_some() && !quantities.is_empty() {
             duration = Some(quantities[0].clone());
        }

        return Ok(Trigger::Periodic {
            interval: interval_val,
            interval_unit,
            duration,
            offset: anchor,
            specific_day,
        });
    }

    Ok(Trigger::StartOf(s.to_string()))
}

// -----------------------------------------------------------------------------
// Property Parsers
// -----------------------------------------------------------------------------

fn parse_value_property(pair: pest::iterators::Pair<Rule>) -> Result<Property, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::valid_values_prop => {
            let line = inner.line_col().0;
            let inner_node = inner.into_inner().next().unwrap();
            match inner_node.as_rule() {
                Rule::valid_values_block => {
                    // Legacy/Standard: list of selectors
                    // Convert selectors to generic statements? Or keep ValidValues hybrid?
                    // AST has ValidValues(Vec<Statement>).
                    // RangeSelector is not a Statement.
                    // But we can wrap it as a generic Command or create a definition statement type.
                    // Or Revert AST to use a specific Enum that covers both?
                    // Property::ValidValues(ValidValuesContent). Enum { Selectors(Vec), Statements(Vec) }.
                    // Simple hack: Wrap RangeSelector in a dummy Statement::Conditional or Expression?
                    // Let's create a new Statement variant: Statement::RangeSelector(RangeSelector)?
                    // No, Statement is user-facing logic.
                    // Let's assume ValidValues(Vec<Statement>) is what we want for POWER.
                    // For legacy `0...5`, we can treat it as `value is between 0...5`.
                    // Which is a Statement::Constraint!
                    // constraint = { expression ~ ... }. "value" is expression.
                    let selectors = parse_valid_values_block(inner_node)?;
                    let mut stmts = Vec::new();
                    for sel in selectors {
                        // Convert RangeSelector to Constraint: "value" <selector>
                        // Actually Statement::Constraint is just a command string in current parser implementation!
                        // "Treat constraints as no-op commands for now" (Line 317 parser.rs).
                        // We should upgrade AST Constraint to be real.
                        // But for now, let's just push a Command("Legacy Range ...").
                        // Or better: Revert AST change and allow `Property::ValidValues` to hold `Vec<AssessmentCase>` (as meaning) OR something else?
                        // The user wants `assess value: ...` (Statements).
                        // Best path: Property::ValidValuesStr(String) (lazy) or generic.

                        // Decision: Wrap it in a Statement::EventProgression (list of cases) where cases have condition=selector, block=empty?
                        // AssessmentCase { condition: sel, block: [] }.
                        // Statement::EventProgression(vec![case]).
                        // This preserves the logic "if matches selector, valid".
                        use crate::ast::{AssessmentCase, Statement, StatementKind};
                        stmts.push(Statement {
                            kind: StatementKind::EventProgression(
                                "value".to_string(),
                                vec![AssessmentCase {
                                    condition: sel,
                                    block: vec![],
                                    line,
                                }],
                            ),
                            line,
                        });
                    }
                    Ok(Property::ValidValues(stmts))
                }
                Rule::inline_valid_values => {
                    let selectors = parse_valid_values_block(inner_node)?; // Reuse block parser as structure is compatible (list of items)
                    let mut stmts = Vec::new();
                    for sel in selectors {
                        use crate::ast::{AssessmentCase, Statement, StatementKind};
                        stmts.push(Statement {
                            kind: StatementKind::EventProgression(
                                "value".to_string(),
                                vec![AssessmentCase {
                                    condition: sel,
                                    block: vec![],
                                    line,
                                }],
                            ),
                            line,
                        });
                    }
                    Ok(Property::ValidValues(stmts))
                }
                _ => {
                    // Flexible block (block_body) -> Statements
                    let stmts = parse_block(inner_node)?;
                    Ok(Property::ValidValues(stmts))
                }
            }
        }
        Rule::meaning_prop => parse_meaning_prop(inner),
        Rule::timeframe_prop => {
            let inner_node = inner.into_inner();
            // timeframe_prop children are timeframe_line+
            let mut all_timeframes = Vec::new();
            for line in inner_node {
                if line.as_rule() == Rule::timeframe_line {
                    let mut selectors = Vec::new();
                    for sel in line.into_inner() {
                        if sel.as_rule() == Rule::timeframe_selector {
                            selectors.push(parse_range_selector(sel)?);
                        }
                    }
                    if !selectors.is_empty() {
                        all_timeframes.push(selectors);
                    }
                }
            }
            Ok(Property::Timeframe(all_timeframes))
        }
        Rule::question_prop => {
            let mut q_inner = inner.into_inner();
            let stmt_or_block = q_inner.next().unwrap();
            
            let action = if stmt_or_block.as_rule() == Rule::block_body {
                 let stmt = stmt_or_block.into_inner().next().unwrap();
                 let act = stmt.into_inner().next().unwrap(); // action
                 let ask = act.into_inner().next().unwrap(); // ask_question
                 parse_ask_question(ask.into_inner())?
            } else if stmt_or_block.as_rule() == Rule::statement {
                 // Direct statement
                 let act = stmt_or_block.into_inner().next().unwrap();
                 let ask = act.into_inner().next().unwrap();
                 parse_ask_question(ask.into_inner())?
            } else {
                 return Ok(Property::Custom("Error".to_string(), "Parse Error".to_string()));
            };
            
            Ok(Property::Question(action))
        }
        Rule::calculation_prop => {
            let block_pair = inner.into_inner().next().unwrap();
            let stmts = parse_block(block_pair)?;
            Ok(Property::Calculation(stmts))
        }

        Rule::inheritance_prop => {
            let mut props_inner = inner.into_inner();
            let ident = parse_identifier_str(props_inner.next().unwrap());
            let mut overrides = Vec::new();

            for prop in props_inner {
                if prop.as_rule() == Rule::value_property {
                    overrides.push(parse_value_property(prop)?);
                }
            }

            let overrides = if overrides.is_empty() { None } else { Some(overrides) };
            Ok(Property::Inheritance(ident, overrides))
        }
        Rule::documentation_prop => {
            let mut doc = None;
            for child in inner.into_inner() {
                if child.as_rule() == Rule::string_literal {
                    doc = Some(parse_string_literal(child));
                    break;
                }
                for grand in child.clone().into_inner() {
                    if grand.as_rule() == Rule::string_literal {
                        doc = Some(parse_string_literal(grand));
                        break;
                    }
                }
                if doc.is_some() {
                    break;
                }
            }
            Ok(Property::Documentation(doc.unwrap_or_default()))
        }
        Rule::unit_ref_prop => { // "unit" is/[:] <unit>
            // inner: unit
            let unit_pair = inner.into_inner().next().unwrap();
            Ok(Property::Unit(parse_unit(unit_pair)?))
        }
        Rule::reuse_prop => parse_reuse_prop(inner),
        Rule::generic_property => {
            let mut props_inner = inner.into_inner();
            let name = parse_identifier_str(props_inner.next().unwrap());
            let mut content_parts: Vec<String> = Vec::new();

            for part in props_inner {
                match part.as_rule() {
                    Rule::property_content | Rule::property_line => {
                        content_parts.push(part.as_str().to_string());
                    }
                    _ => {}
                }
            }

            let content = content_parts.join("").trim().to_string();
            Ok(Property::Custom(name, content))
        }
        _ => Ok(Property::Custom(inner.as_str().to_string(), "".to_string())),
    }
}

fn parse_reuse_prop(pair: pest::iterators::Pair<Rule>) -> Result<Property, ParseError> {
    let inner = pair.into_inner().next().unwrap(); // reuse_stmt
    // reuse_stmt = { "reuse" ~ "period" ~ "of" ~ "value" ~ "is" ~ quantity ~ "."? ~ NEWLINE* }
    
    let mut qty_pair = None;
    for child in inner.into_inner() {
        if child.as_rule() == Rule::quantity {
            qty_pair = Some(child);
            break;
        }
    }
    
    if let Some(pair) = qty_pair {
        let (val, unit) = parse_quantity_pair(pair)?;
        Ok(Property::Reuse(val, unit))
    } else {
        Err(ParseError::ValidationError("Missing quantity in reuse statement".to_string()))
    }
}

fn parse_meaning_prop(pair: pest::iterators::Pair<Rule>) -> Result<Property, ParseError> {
    let mut valid_meanings = Vec::new();
    let mut cases = Vec::new();

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::identifier => {
                // Required "meaning of <id>" target - currently unused.
            }
            Rule::meaning_item_block => {
                for inner in child.into_inner() {
                    match inner.as_rule() {
                        Rule::valid_meanings_prop => {
                            valid_meanings.extend(parse_valid_meanings(inner));
                        }
                        Rule::meaning_assess_block => {
                            for block_child in inner.into_inner() {
                                if block_child.as_rule() == Rule::assessment_case {
                                    cases.extend(parse_assessment_case(block_child)?);
                                }
                            }
                        }
                        Rule::assessment_case => {
                            cases.extend(parse_assessment_case(inner)?);
                        }
                        _ => {}
                    }
                }
            }
            Rule::valid_meanings_prop => {
                valid_meanings.extend(parse_valid_meanings(child));
            }
            Rule::meaning_assess_block => {
                for inner in child.into_inner() {
                    if inner.as_rule() == Rule::assessment_case {
                        cases.extend(parse_assessment_case(inner)?);
                    }
                }
            }
            Rule::assessment_case => {
                cases.extend(parse_assessment_case(child)?);
            }
            _ => {}
        }
    }

    Ok(Property::Meaning(crate::ast::MeaningDef {
        cases,
        valid_meanings,
    }))
}

fn parse_valid_meanings(pair: pest::iterators::Pair<Rule>) -> Vec<String> {
    let mut meanings = Vec::new();
    let mut stack = vec![pair];
    while let Some(node) = stack.pop() {
        match node.as_rule() {
            Rule::identifier => meanings.push(parse_identifier_str(node)),
            _ => {
                for child in node.into_inner() {
                    stack.push(child);
                }
            }
        }
    }
    meanings
}

fn parse_valid_values_block(
    pair: pest::iterators::Pair<Rule>,
) -> Result<Vec<RangeSelector>, ParseError> {
    let mut selectors = Vec::new();
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::safe_range_item => {
                // safe_range_item = { range_selector ~ !":" }
                let inner = p.into_inner().next().unwrap();
                selectors.push(parse_range_selector(inner)?);
            }
            Rule::valid_values_line => {
                for item in p.into_inner() {
                    if item.as_rule() == Rule::safe_range_item {
                        let inner = item.into_inner().next().unwrap();
                        selectors.push(parse_range_selector(inner)?);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(selectors)
}

fn parse_assessment_case(
    pair: pest::iterators::Pair<Rule>,
) -> Result<Vec<AssessmentCase>, ParseError> {
    let mut inner = pair.into_inner();
    let selector_list = inner.next().unwrap();
    let block_rule = inner.next().unwrap();

    let mut selectors = Vec::new();
    for sel in selector_list.into_inner() {
        let line = sel.as_span().start_pos().line_col().0;
        match sel.as_rule() {
            Rule::range_selector => selectors.push((parse_range_selector(sel)?, line)),

            _ => {
                return Err(ParseError::UnknownRule(format!(
                    "Unexpected selector rule: {:?}",
                    sel.as_rule()
                )));
            }
        }
    }

    let block = parse_block(block_rule)?;

    let mut cases = Vec::new();
    for (cond, line) in selectors {
        cases.push(AssessmentCase {
            condition: cond,
            block: block.clone(),
            line,
        });
    }

    Ok(cases)
}

fn parse_range_selector(pair: pest::iterators::Pair<Rule>) -> Result<RangeSelector, ParseError> {
    let s = pair.as_str().trim().to_string();
    
    // Handle "Not enough data" literal directly
    // Normalize whitespace to single space to handle newlines/tabs allowed by grammar
    let normalized = s.split_whitespace().collect::<Vec<&str>>().join(" ");
    if normalized == "Not enough data" {
        return Ok(RangeSelector::NotEnoughData);
    }

    let mut inner = pair.into_inner();
    
    // Check if empty (should not happen if grammar enforces)
    let first = if let Some(p) = inner.next() { p } else {
         // Maybe just string?
         return Ok(RangeSelector::Equals(Expression::Literal(Literal::String(s))));
    };

    // If first is expression
    if first.as_rule() == Rule::expression {
         let e1 = parse_expression(first)?;
         if let Some(second) = inner.next() {
              if second.as_rule() == Rule::expression {
                   let e2 = parse_expression(second)?;
                   return Ok(RangeSelector::Range(e1, e2));
              }
         }
         return Ok(RangeSelector::Equals(e1));
    }

    if first.as_rule() == Rule::identifier || first.as_rule() == Rule::angled_identifier {
        let ident = parse_identifier_str(first);
        return Ok(RangeSelector::Equals(Expression::Variable(ident)));
    }

    if first.as_rule() == Rule::string_literal {
        let s = parse_string_literal(first);
        return Ok(RangeSelector::Equals(Expression::Literal(Literal::String(s))));
    }
    
    // Fallback?
    Ok(RangeSelector::Equals(Expression::Literal(Literal::String(s))))
}


// -----------------------------------------------------------------------------
// Statement Parsers
// -----------------------------------------------------------------------------

fn parse_block(pair: pest::iterators::Pair<Rule>) -> Result<Block, ParseError> {
    let mut stmts = Vec::new();
    for stmt in pair.into_inner() {
        let extra_stmts = extract_question_modifier_block(&stmt)?;
        stmts.push(parse_statement(stmt)?);
        stmts.extend(extra_stmts);
    }
    Ok(stmts)
}

fn extract_question_modifier_block(stmt: &pest::iterators::Pair<Rule>) -> Result<Vec<Statement>, ParseError> {
    let mut extra = Vec::new();
    let mut inner_iter = stmt.clone().into_inner();
    let inner = match inner_iter.next() {
        Some(i) => i,
        None => return Ok(extra),
    };

    if inner.as_rule() == Rule::action {
        for action_child in inner.into_inner() {
            if action_child.as_rule() == Rule::question_modifier {
                let mut stack = vec![action_child];
                while let Some(node) = stack.pop() {
                    if node.as_rule() == Rule::block_body {
                        extra.extend(parse_block(node)?);
                        continue;
                    }
                    for child in node.into_inner() {
                        stack.push(child);
                    }
                }
            }
        }
    }

    Ok(extra)
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Result<Statement, ParseError> {
    let span = pair.as_span();
    let line = span.start_pos().line_col().0;

    let mut inner_iter = pair.into_inner();
    let inner = match inner_iter.next() {
        Some(i) => i,
        None => {
            return Ok(Statement {
                kind: StatementKind::NoOp,
                line,
            });
        }
    };
    let kind = match inner.as_rule() {
        Rule::assignment => {
            let mut a_inner = inner.into_inner();
            let target_pair = a_inner.next().unwrap();
            let target = parse_multi_word_identifier(target_pair);
            let expr = parse_expression(a_inner.next().unwrap())?;
            StatementKind::Assignment(Assignment {
                target,
                expression: expr,
            })
        }
        Rule::meaning_assignment => {
            // rule: "meaning" ~ "of" ~ "value" ~ "=" ~ expression
            // inner contains only expression
            let expr_pair = inner.into_inner().next().unwrap();
            let expr = parse_expression(expr_pair)?;
            StatementKind::Assignment(Assignment {
                target: "meaning of value".to_string(),
                expression: expr,
            })
        }
        Rule::action => match parse_action(inner)? {
            act => StatementKind::Action(act),
        },
        Rule::conditional => {
            let mut c_inner = inner.into_inner();
            let condition_pair = c_inner.next().unwrap();
            let target = if condition_pair.as_rule() == Rule::confidence_target {
                let ident_pair = condition_pair.into_inner().next().unwrap();
                let ident = parse_identifier_str(ident_pair);
                ConditionalTarget::Confidence(ident)
            } else {
                let expr = parse_expression(condition_pair)?;
                ConditionalTarget::Expression(expr)
            };

            let mut cases = Vec::new();

            for p in c_inner {
                match p.as_rule() {
                    Rule::assessment_case => cases.extend(parse_assessment_case(p)?),
                    _ => {}
                }
            }
            StatementKind::Conditional(Conditional {
                condition: target,
                cases,
            })
        }
        Rule::context_block => {
            let mut items = Vec::new();
            let mut statements = Vec::new();
            let c_inner = inner.into_inner();

            for p in c_inner {
                match p.as_rule() {
                    Rule::context_item => {
                        let s = p.as_str().trim().to_string();
                        let i_inner = p.into_inner();

                        if s.starts_with("timeframe") {
                            for child in i_inner {
                                if child.as_rule() == Rule::timeframe_selector {
                                    items
                                        .push(ContextItem::Timeframe(parse_range_selector(child)?));
                                    break;
                                }
                            }
                        } else if s.starts_with("data") {
                            for child in i_inner {
                                if child.as_rule() == Rule::identifier
                                    || child.as_rule() == Rule::string_literal
                                {
                                    items.push(ContextItem::Data(parse_identifier_str(child)));
                                    break;
                                }
                            }
                        } else if s.starts_with("value filter") {
                            for child in i_inner {
                                if child.as_rule() == Rule::assessment_case {
                                    let ac_vec = parse_assessment_case(child)?;
                                    if let Some(a) = ac_vec.into_iter().next() {
                                        items.push(ContextItem::ValueFilter(a));
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    Rule::statement => {
                        statements.push(parse_statement(p)?);
                    }
                    _ => {}
                }
            }
            StatementKind::ContextBlock(ContextBlock { items, statements })
        }
        Rule::timeframe_block => {
             let mut tf_inner = inner.into_inner().peekable();
             let mut for_analysis = false;
             let mut constraints = Vec::new();
             let mut stmts = Vec::new();

             while let Some(pair) = tf_inner.peek() {
                 match pair.as_rule() {
                     Rule::for_analysis_flag => {
                         for_analysis = true;
                         tf_inner.next();
                     }
                     Rule::constraint_operator => {
                         let op = tf_inner.next().unwrap().as_str().to_string();
                         let range_pair = tf_inner.next().ok_or_else(|| {
                             ParseError::ValidationError(
                                 "Missing range selector for timeframe constraint".to_string(),
                             )
                         })?;
                         let range = parse_range_selector(range_pair)?;
                         constraints.push((op, range));
                     }
                     Rule::statement => {
                         stmts.push(parse_statement(tf_inner.next().unwrap())?);
                     }
                     _ => {
                         tf_inner.next();
                     }
                 }
             }

             StatementKind::Timeframe(crate::ast::TimeframeBlock {
                 for_analysis,
                 constraints,
                 block: stmts,
             })
        }

        Rule::documentation_prop => StatementKind::Command("Documentation".to_string()),
        Rule::constraint => {
            let mut c_inner = inner.into_inner();
            let expr = parse_expression(c_inner.next().unwrap())?;
            let op = c_inner.next().unwrap().as_str().to_string();
            let sel = parse_range_selector(c_inner.next().unwrap())?;
            StatementKind::Constraint(expr, op, sel)
        }
        _ => StatementKind::Command("Unknown".to_string()),
    };
    Ok(Statement { kind, line })
}

fn parse_action(pair: pest::iterators::Pair<Rule>) -> Result<Action, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::show_message => Ok(parse_show_message(inner.into_inner())?),
        Rule::say_message => Ok(parse_show_message(inner.into_inner())?),
        Rule::ask_question => Ok(parse_ask_question(inner.into_inner())?),
        Rule::send_info => {
            let mut pairs = inner.into_inner();
            let msg = parse_string_literal(pairs.next().unwrap());
            let mut exprs = vec![];
            for p in pairs {
                exprs.push(parse_expression(p)?);
            }
            Ok(Action::SendInfo(msg, exprs))
        }
        Rule::listen_for => {
            let mut pairs = inner.into_inner();
            let ident = parse_identifier_str(pairs.next().unwrap());
            Ok(Action::ListenFor(ident))
        }
        Rule::simple_command => {
            let fallback = inner.as_str().trim_end_matches('.').trim().to_string();
            let mut s_inner = inner.into_inner();
            if let Some(ident) = s_inner.next() {
                Ok(Action::ListenFor(parse_identifier_str(ident)))
            } else {
                Ok(Action::ListenFor(fallback))
            }
        }
        Rule::question_modifier => {
            let mut q_check = inner.clone().into_inner();
            if let Some(first) = q_check.next() {
                if first.as_rule() == Rule::validate_modifier {
                    return parse_validate_modifier(first);
                }
            }
            let raw = inner.as_str().trim().trim_end_matches('.');

            if raw.starts_with("type of question is") {
                if let Some(lit) = inner.clone().into_inner().find(|p| p.as_rule() == Rule::string_literal) {
                    let t = parse_string_literal(lit);
                    return Ok(Action::Configure(QuestionConfig::Type(t)));
                }
            }

            if raw.starts_with("style of question is") {
                if let Some(ident) = inner.clone().into_inner().find(|p| p.as_rule() == Rule::identifier) {
                    let s = parse_identifier_str(ident);
                    return Ok(Action::Configure(QuestionConfig::Style(s)));
                }
            }
            // detailed check for vas_block
            if let Some(vas_pair) = inner.clone().into_inner().find(|p| p.as_rule() == Rule::vas_block) {
                 let mut best_val = 0.0;
                 let mut worst_val = 10.0; // defaults?
                 let mut best_lbl = None;
                 let mut worst_lbl = None;

                 for p in vas_pair.into_inner() {
                     match p.as_rule() {
                         Rule::best_value_def => {
                             let inner = p.into_inner().next().unwrap(); // quantity or number
                             let expr = parse_expression(inner)?; // parse as expression to preserve value
                             best_val = match expr {
                                 Expression::Literal(Literal::Number(n, _)) => n,
                                 Expression::Literal(Literal::Quantity(n, _, _)) => n,
                                 _ => 0.0,
                             };
                         }
                         Rule::worst_value_def => {
                             let inner = p.into_inner().next().unwrap();
                             let expr = parse_expression(inner)?;
                             worst_val = match expr {
                                 Expression::Literal(Literal::Number(n, _)) => n,
                                 Expression::Literal(Literal::Quantity(n, _, _)) => n,
                                 _ => 10.0,
                             };
                         }
                         Rule::best_label_def => {
                             let inner = p.into_inner().next().unwrap(); // string_literal
                             best_lbl = Some(parse_string_literal(inner));
                         }
                         Rule::worst_label_def => {
                             let inner = p.into_inner().next().unwrap(); // string_literal
                             worst_lbl = Some(parse_string_literal(inner));
                         }
                         _ => {}
                     }
                 }

                 return Ok(Action::Configure(QuestionConfig::VisualAnalogScale(VasDef {
                     best_value: best_val,
                     best_label: best_lbl,
                     worst_value: worst_val,
                     worst_label: worst_lbl,
                 })));
            }

            let s = inner.as_str().trim().to_string();
            Ok(Action::Configure(QuestionConfig::Generic(s)))
        }
        Rule::start_period => {
            // start_period = { "start" ~ identifier ... }
            Ok(Action::StartPeriod)
        }
        _ => Err(ParseError::UnknownRule(format!("{:?}", inner.as_rule()))),
    }
}

// -----------------------------------------------------------------------------
// Action Specific Parsers
// -----------------------------------------------------------------------------

fn parse_show_message(pairs: pest::iterators::Pairs<Rule>) -> Result<Action, ParseError> {
    let mut message_parts = Vec::new();
    let mut statements = Vec::new();

    fn push_message_expiration(
        pair: pest::iterators::Pair<Rule>,
        statements: &mut Vec<Statement>,
    ) -> Result<(), ParseError> {
        let line = pair.as_span().start_pos().line_col().0;
        let mut inner = pair.into_inner();
        let value_pair = inner.next().unwrap();
        let value_inner = if value_pair.as_rule() == Rule::message_expiration_value {
            value_pair.into_inner().next().unwrap()
        } else {
            value_pair
        };
        let expr = parse_expression(value_inner)?;
        let rs = RangeSelector::Equals(expr);
        statements.push(Statement {
            kind: StatementKind::Action(Action::MessageExpiration(rs)),
            line,
        });
        Ok(())
    }

    fn handle_pair(
        pair: pest::iterators::Pair<Rule>,
        message_parts: &mut Vec<Expression>,
        statements: &mut Vec<Statement>,
    ) -> Result<(), ParseError> {
        match pair.as_rule() {
            Rule::expression => {
                message_parts.push(parse_expression(pair)?);
            }
            Rule::message_expiration => {
                push_message_expiration(pair, statements)?;
            }
            Rule::message_block | Rule::message_block_line | Rule::message_property_block => {
                for child in pair.into_inner() {
                    handle_pair(child, message_parts, statements)?;
                }
            }
            Rule::message_property => {
                for child in pair.into_inner() {
                    handle_pair(child, message_parts, statements)?;
                }
            }
            Rule::string_literal => {
                let s = pair.as_str().trim_matches('"').to_string();
                message_parts.push(Expression::Literal(Literal::String(s)));
            }
            _ => {}
        }
        Ok(())
    }

    for p in pairs {
        handle_pair(p, &mut message_parts, &mut statements)?;
    }

    Ok(Action::ShowMessage(
        message_parts,
        if statements.is_empty() {
            None
        } else {
            Some(statements)
        },
    ))
}

fn parse_ask_question(pairs: pest::iterators::Pairs<Rule>) -> Result<Action, ParseError> {
    let mut subject = String::new();
    let mut statements = vec![];

    for p in pairs {

        match p.as_rule() {
            Rule::multi_word_identifier => {
                subject = parse_multi_word_identifier(p);
            }
            Rule::identifier | Rule::angled_identifier => {
                subject = parse_identifier_str(p);
            }
            Rule::string_literal => {
                subject = p.as_str().trim_matches('"').to_string();
            }
            Rule::statement => {
                let mut extra_stmts = Vec::new();

                let mut stmt_inner = p.clone().into_inner();
                if let Some(stmt_first) = stmt_inner.next() {
                    if stmt_first.as_rule() == Rule::action {
                        for action_child in stmt_first.into_inner() {
                            if action_child.as_rule() == Rule::question_modifier {
                                for qm_child in action_child.into_inner() {
                                    if qm_child.as_rule() == Rule::flexible_block {
                                        if let Some(block_body) = qm_child.into_inner().next() {
                                            extra_stmts.extend(parse_block(block_body)?);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                statements.push(parse_statement(p)?);
                statements.extend(extra_stmts);
            }
            Rule::block_body => {
                statements.extend(parse_block(p)?);
            }
            _ => {}
        }
    }

    let opt_stmts = if statements.is_empty() {
        None
    } else {
        Some(statements)
    };

    Ok(Action::AskQuestion(subject, opt_stmts))
}

fn parse_validate_modifier(pair: pest::iterators::Pair<Rule>) -> Result<Action, ParseError> {
    let mut inner = pair.into_inner();
    // first child is validation_mode ("once" | "twice")
    let mode_pair = inner.next().unwrap();
    let mode = match mode_pair.as_str() {
        "twice" => ValidationMode::Twice,
        _ => ValidationMode::Once,
    };

    let mut timeout = None;
    for p in inner {
        if p.as_rule() == Rule::quantity {
            let q_expr = parse_expression(p)?;
            if let Expression::Literal(Literal::Quantity(v, u, _)) = q_expr {
                timeout = Some((v, u));
            }
        }
    }
    Ok(Action::ValidateAnswer(mode, timeout))
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn parse_string_literal(pair: pest::iterators::Pair<Rule>) -> String {
    pair.as_str().trim_matches('"').to_string()
}

fn parse_multi_word_identifier(pair: pest::iterators::Pair<Rule>) -> String {
    let mut s = String::new();
    for part in pair.into_inner() {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(&parse_identifier_str(part));
    }
    s
}

fn parse_identifier_str(pair: pest::iterators::Pair<Rule>) -> String {
    if pair.as_rule() == Rule::identifier {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::angled_identifier => {
                let s = inner.as_str();
                s[1..s.len() - 1].to_string()
            }
            Rule::raw_identifier => inner.as_str().to_string(),
            Rule::string_literal => inner.as_str().trim_matches('"').to_string(),
            _ => inner.as_str().to_string(),
        }
    } else if pair.as_rule() == Rule::raw_identifier {
        pair.as_str().to_string()
    } else if pair.as_rule() == Rule::angled_identifier {
        let s = pair.as_str();
        s[1..s.len() - 1].to_string()
    } else if pair.as_rule() == Rule::string_literal {
        let s = pair.as_str();
        s[1..s.len() - 1].to_string()
    } else {
        pair.as_str().to_string()
    }
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
    // println!("DEBUG: parse_expression rule: {:?}, str: {:?}", pair.as_rule(), pair.as_str());
    match pair.as_rule() {
        Rule::expression => {
            let mut pairs = pair.into_inner();
            let mut expr = parse_expression(pairs.next().unwrap())?;

            while let Some(op) = pairs.next() {
                let s = op.as_str().to_string(); // op
                let right_pair = pairs.next().unwrap();
                let right = parse_expression(right_pair)?;

                // TODO: Handle precedence properly. For now, flat left-associative.
                expr = Expression::Binary(Box::new(expr), s, Box::new(right));
            }
            Ok(expr)
        }
        Rule::term => {
            let mut pairs = pair.into_inner();
            let first = pairs.next().unwrap();

            if first.as_rule() == Rule::quantity {
                if let Some(second) = pairs.peek() {
                    if second.as_rule() == Rule::relative_time_modifier {
                        let _modifier = pairs.next().unwrap();

                        // Parse quantity by calling parse_expression (it handles Rule::quantity)
                        let qty_expr = parse_expression(first)?;
                        let (val, unit) = match qty_expr {
                            Expression::Literal(Literal::Quantity(v, u, _)) => (v, u),
                            _ => unreachable!("Parsed quantity but got distinct expression"),
                        };

                        let direction = match second.as_str() {
                            "ago" => RelativeDirection::Ago,
                            "from now" => RelativeDirection::FromNow,
                            _ => RelativeDirection::Ago,
                        };
                        return Ok(Expression::RelativeTime(val, unit, direction));
                    }
                }
            }
            parse_expression(first)
        }
        Rule::meaning_of_expr => {
            let mut inner = pair.into_inner();
            let ident = parse_identifier_str(inner.next().unwrap());
            Ok(Expression::MeaningOf(ident))
        }
        Rule::number => {
            let s = pair.as_str().trim();
            let val = s.parse::<f64>().unwrap_or(0.0);
            let precision = if let Some(idx) = s.find('.') {
                Some(s.len() - idx - 1)
            } else {
                None
            };
            Ok(Expression::Literal(Literal::Number(val, precision)))
        }
        Rule::string_literal => Ok(Expression::Literal(Literal::String(
            pair.as_str().trim_matches('"').to_string(),
        ))),
        Rule::quantity => {
            let mut q_inner = pair.into_inner();
            let num_pair = q_inner.next().unwrap();
            let s = num_pair.as_str().trim();
            let val = s.parse::<f64>().unwrap_or(0.0);
            let precision = if let Some(idx) = s.find('.') {
                Some(s.len() - idx - 1)
            } else {
                None
            };
            let unit_pair = q_inner.next().unwrap();
            let unit = parse_unit(unit_pair)?;
            Ok(Expression::Literal(Literal::Quantity(val, unit, precision)))
        }
        Rule::statistical_func => {
            let s = pair.as_str();
            let mut inner = pair.into_inner();
            let ident_pair = inner.next().unwrap();

            let ident = if ident_pair.as_rule() == Rule::multi_word_identifier {
                parse_multi_word_identifier(ident_pair)
            } else {
                parse_identifier_str(ident_pair)
            };

            if s.starts_with("count of") {
                let filter = if let Some(term_pair) = inner.next() {
                    Some(Box::new(parse_expression(term_pair)?))
                } else {
                    None
                };
                Ok(Expression::Statistical(StatisticalFunc::CountOf(
                    ident, filter,
                )))
            } else if s.starts_with("min of") {
                Ok(Expression::Statistical(StatisticalFunc::MinOf(ident)))
            } else if s.starts_with("max of") {
                Ok(Expression::Statistical(StatisticalFunc::MaxOf(ident)))
            } else if s.starts_with("trend of") {
                Ok(Expression::Statistical(StatisticalFunc::TrendOf(ident)))
            } else if s.starts_with("average of") {
                // Expect quantity next
                let q_pair = inner.next().unwrap();
                let q_expr = parse_expression(q_pair)?;
                Ok(Expression::Statistical(StatisticalFunc::AverageOf(
                    ident,
                    Box::new(q_expr),
                )))
            } else {
                Ok(Expression::Variable(ident))
            }
        }
        Rule::identifier => {
            let s = parse_identifier_str(pair);
            Ok(Expression::Variable(s))
        }
        Rule::multi_word_identifier => Ok(Expression::Variable(parse_multi_word_identifier(pair))),
        Rule::date_literal | Rule::datetime_literal => {
            let value = pair.as_str().to_string();
            validate_date_literal(&value)?;
            Ok(Expression::Literal(Literal::Date(value)))
        }
        Rule::date_diff => {
            let mut inner = pair.into_inner();
            let unit_pair = inner.next().unwrap();
            let unit = parse_date_diff_unit(unit_pair.as_str())?;
            let start = parse_expression(inner.next().unwrap())?;
            let end = parse_expression(inner.next().unwrap())?;
            Ok(Expression::DateDiff(
                unit,
                Box::new(start),
                Box::new(end),
            ))
        }
        Rule::time_literal => Ok(Expression::Literal(Literal::TimeOfDay(
            pair.as_str().to_string(),
        ))),
        Rule::time_indication => {
            let s = pair.as_str();
            if s == "now" {
                Ok(Expression::Variable("now".to_string()))
            } else {
                 Ok(Expression::Literal(Literal::String(s.to_string())))
            }
        }
        _ => Ok(Expression::Literal(Literal::String(
            pair.as_str().to_string(),
        ))),
    }
}

fn parse_date_diff_unit(s: &str) -> Result<Unit, ParseError> {
    match s {
        "year" | "years" => Ok(Unit::Year),
        "month" | "months" => Ok(Unit::Month),
        "day" | "days" => Ok(Unit::Day),
        "hour" | "hours" => Ok(Unit::Hour),
        "minute" | "minutes" => Ok(Unit::Minute),
        _ => Err(ParseError::ValidationError(format!(
            "Invalid date diff unit '{}'",
            s
        ))),
    }
}

fn validate_date_literal(value: &str) -> Result<(), ParseError> {
    if NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%d %-H:%M").is_ok()
    {
        return Ok(());
    }
    if NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok() {
        return Ok(());
    }
    Err(ParseError::ValidationError(format!(
        "Invalid date/time literal '{}'",
        value
    )))
}



fn parse_unit(pair: pest::iterators::Pair<Rule>) -> Result<Unit, ParseError> {
    match pair.as_str() {
        "mg" | "milligram" | "milligrams" => Ok(Unit::Milligram),
        "kg" | "kilogram" | "kilograms" => Ok(Unit::Kilogram),
        "g" | "gram" | "grams" => Ok(Unit::Gram),
        "lb" | "pound" | "pounds" => Ok(Unit::Pound),
        "oz" | "ounce" | "ounces" => Ok(Unit::Ounce),
        "m" | "meter" | "meters" => Ok(Unit::Meter),
        "cm" | "centimeter" | "centimeters" => Ok(Unit::Centimeter),
        "mm" | "millimeter" | "millimeters" => Ok(Unit::Millimeter),
        "km" | "kilometer" | "kilometers" => Ok(Unit::Kilometer),
        "inch" | "inches" => Ok(Unit::Inch),
        "foot" | "feet" => Ok(Unit::Foot),
        "mile" | "miles" => Ok(Unit::Mile),
        "l" | "liter" | "liters" => Ok(Unit::Liter),
        "ml" | "milliliter" | "milliliters" => Ok(Unit::Milliliter),
        "fl oz" | "fluid ounce" | "fluid ounces" => Ok(Unit::FluidOunce),
        "gal" | "gallon" | "gallons" => Ok(Unit::Gallon),
        "°C" | "celsius" => Ok(Unit::Celsius),
        "°F" | "fahrenheit" => Ok(Unit::Fahrenheit),
        "mmHg" | "millimeter of mercury" => Ok(Unit::MillimeterOfMercury),
        "bpm" => Ok(Unit::Bpm),
        "mg/dL" => Ok(Unit::MgPerDl),
        "mmol/L" => Ok(Unit::MmolPerL),
        "%" | "percent" => Ok(Unit::Percent),
        "year" | "years" => Ok(Unit::Year),
        "month" | "months" => Ok(Unit::Month),
        "week" | "weeks" => Ok(Unit::Week),
        "day" | "days" => Ok(Unit::Day),
        "hour" | "hours" => Ok(Unit::Hour),
        "minute" | "minutes" => Ok(Unit::Minute),
        "second" | "seconds" => Ok(Unit::Second),
        _ => {
            let s = pair.as_str();
            let name = if s.starts_with('<') && s.ends_with('>') {
                s[1..s.len() - 1].to_string()
            } else {
                s.to_string()
            };
            Ok(Unit::Custom(name))
        }
    }
}
