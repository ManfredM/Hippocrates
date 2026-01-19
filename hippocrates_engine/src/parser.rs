use crate::ast::*;
use crate::domain::*;
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
}

pub fn parse_plan(input: &str) -> Result<Plan, ParseError> {
    let processed = preprocess_indentation(input);
    println!("DEBUG: Preprocessed Input:\n{}", processed);
    let mut pairs = HippocratesParser::parse(Rule::file, &processed)?;
    let mut definitions = Vec::new();

    let root = pairs.next().unwrap();
    for pair in root.into_inner() {
        match pair.as_rule() {
            Rule::definition => {
                let inner = pair.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::value_definition => {
                        definitions.push(Definition::Value(parse_value_def(inner)?))
                    }
                    Rule::period_definition => {
                        definitions.push(Definition::Period(parse_period_def(inner)?))
                    }
                    Rule::plan_definition => {
                        definitions.push(Definition::Plan(parse_plan_def(inner)?))
                    }
                    Rule::drug_definition => {
                        definitions.push(Definition::Drug(parse_drug_def(inner)?))
                    }
                    Rule::addressee_definition => {
                        definitions.push(Definition::Addressee(parse_addressee_def(inner)?))
                    }
                    Rule::context_definition => {
                        definitions.push(Definition::Context(parse_context_def(inner)?))
                    }
                    _ => (),
                }
            }
            Rule::EOI => (),
            _ => (),
        }
    }

    Ok(Plan { definitions })
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
        "a time indication" => ValueType::TimeIndication,
        "a period" => ValueType::Period,
        "a plan" => ValueType::Plan,
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
    let mut inner = pair.into_inner();
    let name = parse_identifier_str(inner.next().unwrap());
    Ok(PeriodDef {
        name,
        timeframes: vec![],
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
            let name = parse_string_literal(e_inner.next().unwrap());
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
                    let i_amt = i_inner
                        .next()
                        .unwrap()
                        .as_str()
                        .trim()
                        .parse::<f64>()
                        .unwrap_or(0.0);
                    let i_unit = parse_unit(i_inner.next().unwrap())?;
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
                    if child.as_rule() == Rule::range_selector {
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
            if child.as_rule() == Rule::multi_word_identifier {
                return Ok(Trigger::ChangeOf(parse_multi_word_identifier(child)));
            }
        }
        // Fallback or Error
        return Ok(Trigger::ChangeOf(s.to_string().replace("change of ", "")));
    } else if s.starts_with("begin of") {
        let inner = pair.into_inner();
        // "begin of" is silent? No, "begin of" is string literal in grammar
        // pair: "begin of" ~ multi_word_identifier
        // Skip "begin of"? Pest usually includes it if string literal?
        // Let's inspect tokens.
        // Actually best to look at inner rules.
        // event_trigger = { "begin of" ~ multi_word_identifier | ... }
        // The literal "begin of" might be token or not depending on if it's explicitly rule. It's not.
        // So inner should contain multi_word_identifier.
        for child in inner {
            if child.as_rule() == Rule::multi_word_identifier {
                return Ok(Trigger::StartOf(parse_multi_word_identifier(child)));
            }
        }
        return Ok(Trigger::StartOf(s.to_string())); // Fallback
    } else if s.starts_with("every") {
        let inner = pair.into_inner();
        let mut quantities = Vec::new();

        for child in inner {
            if child.as_rule() == Rule::quantity {
                quantities.push(parse_quantity_pair(child)?);
            }
        }

        let mut interval_val = 1.0;
        let mut interval_unit = Unit::Second;
        let mut duration = None;

        if let Some((v, u)) = quantities.get(0) {
            interval_val = *v;
            interval_unit = u.clone();
        }
        if let Some((v, u)) = quantities.get(1) {
            duration = Some((*v, u.clone()));
        }

        return Ok(Trigger::Periodic {
            interval: interval_val,
            interval_unit,
            duration,
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
                                }],
                            ),
                            line: 0,
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
                                }],
                            ),
                            line: 0,
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
        Rule::meaning_prop => {
            let cases = parse_assessment_cases(inner.into_inner())?;
            Ok(Property::Meaning(cases))
        }
        Rule::timeframe_prop => {
            let inner_node = inner.into_inner();
            // timeframe_prop children are timeframe_line+
            let mut all_timeframes = Vec::new();
            for line in inner_node {
                if line.as_rule() == Rule::timeframe_line {
                    let mut selectors = Vec::new();
                    for sel in line.into_inner() {
                        if sel.as_rule() == Rule::range_selector {
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
        Rule::reuse_prop => Ok(Property::Reuse("Todo".to_string())),
        Rule::inheritance_prop => {
            let ident = parse_identifier_str(inner.into_inner().next().unwrap());
            Ok(Property::Inheritance(ident, None))
        }
        Rule::documentation_prop => Ok(Property::Documentation("Todo".to_string())),
        _ => Ok(Property::Custom(inner.as_str().to_string(), "".to_string())),
    }
}

fn parse_assessment_cases(
    pairs: pest::iterators::Pairs<Rule>,
) -> Result<Vec<AssessmentCase>, ParseError> {
    let mut cases = Vec::new();
    for pair in pairs {
        if pair.as_rule() == Rule::assessment_case {
            cases.extend(parse_assessment_case(pair)?);
        }
    }
    Ok(cases)
}

fn parse_valid_values_block(
    pair: pest::iterators::Pair<Rule>,
) -> Result<Vec<RangeSelector>, ParseError> {
    let mut selectors = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::safe_range_item {
            // safe_range_item = { range_selector ~ !":" }
            let inner = p.into_inner().next().unwrap();
            selectors.push(parse_range_selector(inner)?);
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
        match sel.as_rule() {
            Rule::range_selector => selectors.push(parse_range_selector(sel)?),
            Rule::condition => selectors.push(parse_condition(sel)?),
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
    for cond in selectors {
        cases.push(AssessmentCase {
            condition: cond,
            block: block.clone(),
        });
    }

    Ok(cases)
}

fn parse_range_selector(pair: pest::iterators::Pair<Rule>) -> Result<RangeSelector, ParseError> {
    let s = pair.as_str().trim().to_string();
    
    // Handle "Not enough data" literal directly
    if s == "Not enough data" {
        return Ok(RangeSelector::NotEnoughData);
    }

    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();


    if first.as_rule() == Rule::compare_op {
        let op_str = first.as_str();
        let op = match op_str {
            ">" => ConditionOperator::GreaterThan,
            ">=" => ConditionOperator::GreaterThanOrEquals,
            "<" => ConditionOperator::LessThan,
            "<=" => ConditionOperator::LessThanOrEquals,
            "=" => ConditionOperator::Equals,
            "!=" => ConditionOperator::NotEquals,
            _ => ConditionOperator::Equals,
        };
        let expr_pair = inner.next().unwrap();
        let expr = parse_expression(expr_pair)?;
        return Ok(RangeSelector::Condition(op, expr));
    }

    if let Some(second) = inner.next() {
        if s.starts_with("between") {
            let e1 = parse_expression(first)?;
            let e2 = parse_expression(second)?;
            return Ok(RangeSelector::Between(e1, e2));
        } else {
            let e1 = parse_expression(first)?;
            let e2 = parse_expression(second)?;
            return Ok(RangeSelector::Range(e1, e2));
        }
    }

    // Fallback for implicitly equality or hacked > support (now handled above)
    // If we are here, it matches single expression
    Ok(RangeSelector::Equals(parse_expression(first)?))
}

// -----------------------------------------------------------------------------
// Statement Parsers
// -----------------------------------------------------------------------------

fn parse_block(pair: pest::iterators::Pair<Rule>) -> Result<Block, ParseError> {
    let mut stmts = Vec::new();
    for stmt in pair.into_inner() {
        stmts.push(parse_statement(stmt)?);
    }
    Ok(stmts)
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
                                if child.as_rule() == Rule::range_selector {
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
            let inner = inner.into_inner();
            let mut items = Vec::new();
            let mut statements = Vec::new();

            for p in inner {
                match p.as_rule() {
                    Rule::range_selector => {
                        items.push(ContextItem::Timeframe(parse_range_selector(p)?));
                    }
                    Rule::statement => {
                        statements.push(parse_statement(p)?);
                    }
                    _ => {}
                }
            }
            StatementKind::ContextBlock(ContextBlock { items, statements })
        }
        Rule::event_progression => {
            // "assess event progression" has no variable target in grammar (it's a literal string in rule)
            // So inner parts are just cases.
            let cases = parse_assessment_cases(inner.into_inner())?;
            StatementKind::EventProgression("event progression".to_string(), cases)
        }
        Rule::documentation_prop => StatementKind::Command("Documentation".to_string()),
        Rule::constraint => StatementKind::Command("Constraint".to_string()),
        _ => StatementKind::Command("Unknown".to_string()),
    };
    Ok(Statement { kind, line })
}

fn parse_action(pair: pest::iterators::Pair<Rule>) -> Result<Action, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::show_message => Ok(parse_show_message(inner.into_inner())?),
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
            // Corrected: use inner.as_str() because inner IS the simple_command (multi_word + dot)
            // inner.as_str() includes the dot. We should trim it.
            let s = inner.as_str().trim_end_matches('.').trim().to_string();
            Ok(Action::ListenFor(s))
        }
        Rule::question_modifier => {
            let s = inner.as_str().trim().to_string();
            Ok(Action::Configure(s))
        }
        Rule::start_period => {
            // start_period = { "start" ~ identifier ... }
            Ok(Action::StartPeriod)
        }
        Rule::message_expiration => {
            let mut pairs = inner.into_inner();
            let rs_pair = pairs.next().unwrap();
            let rs = parse_range_selector(rs_pair)?;
            Ok(Action::MessageExpiration(rs))
        }
        _ => Err(ParseError::UnknownRule(format!("{:?}", inner.as_rule()))),
    }
}

// -----------------------------------------------------------------------------
// Action Specific Parsers
// -----------------------------------------------------------------------------

fn parse_show_message(pairs: pest::iterators::Pairs<Rule>) -> Result<Action, ParseError> {
    // show_message = { "show message" ~ ("to" ~ ("patient" | "physician"))? ~ (expression | NEWLINE)+ ~ flexible_block? ~ "."? }
    // Skip "to patient" if present? No, pest pairs only contains non-silent rules.
    // "to" etc are effectively keywords but not silent rules in grammar?
    // Grammar: "show message" ...
    // Pairs iterator iterates over children.
    // My grammar for show_message doesn't have named rules for "to patient", just literals. Literals aren't produced unless atomic?
    // Actually, string literals in rules *are* produced if not silent _
    // But typically we look at relevant internal rules.
    // Pairs will contain: expression, expression... and optionally flexible_block statements.

    // We need to iterate and assume anything that is `expression` is message content.
    // Anything that is `statement` (from flexible block) is a sub-statement.

    // Actually, flexible_block is _{ (":" ~ ... block_body) ... }
    // block_body = { statement+ }
    // So pairs will contain `expression`s and then `statement`s.
    // But `expression` matches `multi_word_identifier` too.

    // Let's iterate.
    // Wait, the first pair logic in old function was:
    // let msg_pair = pairs.next().unwrap(); (assumed string literal)

    // New logic:
    let mut message_parts = Vec::new();
    let mut statements = Vec::new(); // If flexible block exists

    for p in pairs {
        match p.as_rule() {
            Rule::expression => {
                message_parts.push(parse_expression(p)?);
            }
            Rule::statement => {
                statements.push(parse_statement(p)?);
            }
            Rule::string_literal => {
                let s = p.as_str().trim_matches('"').to_string();
                message_parts.push(Expression::Literal(Literal::String(s)));
            }
            _ => {} // Ignore keywords or unknown tokens if any
        }
    }

    // Concatenate message parts into a single string formatting structure?
    // Action::ShowMessage expects (String, Option<Vec<Statement>>).
    // This is a breaking change for Action::ShowMessage if it expects a single String.
    // I should change Action::ShowMessage to (Vec<Expression>, ...) or hack it.
    // But refactoring AST is risky if other things use it.
    // Let's check send_info. It uses (String, Vec<Expression>).
    // User wants complex output.
    // I will refactor Action::ShowMessage to (Vec<Expression>, Option<Vec<Statement>>).
    // Or I'll just change ParseShowMessage to use SendInfo action?
    // No, AST has ShowMessage.

    // Changing AST Action::ShowMessage check.
    // I already checked AST, it was (String, Option...).
    // I MUST update AST to support dynamic message.

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
        println!("DEBUG: parse_ask_question rule: {:?}", p.as_rule());
        match p.as_rule() {
            Rule::multi_word_identifier => {
                subject = parse_multi_word_identifier(p);
            }
            Rule::string_literal => {
                subject = p.as_str().trim_matches('"').to_string();
            }
            Rule::statement => {
                statements.push(parse_statement(p)?);
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
                            Expression::Literal(Literal::Quantity(v, u)) => (v, u),
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
        Rule::number => {
            let val = pair.as_str().trim().parse::<f64>().unwrap_or(0.0);
            Ok(Expression::Literal(Literal::Number(val)))
        }
        Rule::string_literal => Ok(Expression::Literal(Literal::String(
            pair.as_str().trim_matches('"').to_string(),
        ))),
        Rule::quantity => {
            let mut q_inner = pair.into_inner();
            let num_pair = q_inner.next().unwrap();
            let val = num_pair.as_str().trim().parse::<f64>().unwrap_or(0.0);
            let unit_pair = q_inner.next().unwrap();
            let unit = parse_unit(unit_pair)?;
            Ok(Expression::Literal(Literal::Quantity(val, unit)))
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
        Rule::multi_word_identifier => Ok(Expression::Variable(parse_multi_word_identifier(pair))),
        Rule::time_literal => Ok(Expression::Literal(Literal::TimeOfDay(
            pair.as_str().to_string(),
        ))),
        _ => Ok(Expression::Literal(Literal::String(
            pair.as_str().to_string(),
        ))),
    }
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> Result<RangeSelector, ParseError> {
    let mut inner = pair.into_inner();
    let left = parse_expression(inner.next().unwrap())?;
    let op = inner.next().unwrap();
    let right = parse_expression(inner.next().unwrap())?;

    let operator = match op.as_str() {
        "is" | "=" => crate::ast::ConditionOperator::Equals,
        "is not" | "!=" => crate::ast::ConditionOperator::NotEquals,
        ">" => crate::ast::ConditionOperator::GreaterThan,
        "<" => crate::ast::ConditionOperator::LessThan,
        ">=" => crate::ast::ConditionOperator::GreaterThanOrEquals,
        "<=" => crate::ast::ConditionOperator::LessThanOrEquals,
        _ => {
            return Err(ParseError::UnknownRule(format!(
                "Unknown op {}",
                op.as_str()
            )));
        }
    };

    Ok(RangeSelector::Comparison(left, operator, right))
}

fn parse_unit(pair: pest::iterators::Pair<Rule>) -> Result<Unit, ParseError> {
    match pair.as_str() {
        "mg" | "milligram" => Ok(Unit::Milligram),
        "kg" | "kilogram" => Ok(Unit::Kilogram),
        "g" | "gram" => Ok(Unit::Gram),
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
        "%" | "percent" => Ok(Unit::Percent),
        "year" | "years" => Ok(Unit::Year),
        "month" | "months" => Ok(Unit::Month),
        "week" | "weeks" => Ok(Unit::Week),
        "day" | "days" => Ok(Unit::Day),
        "hour" | "hours" => Ok(Unit::Hour),
        "minute" | "minutes" => Ok(Unit::Minute),
        "second" | "seconds" => Ok(Unit::Second),
        _ => Err(ParseError::UnknownUnit(pair.as_str().to_string())),
    }
}
