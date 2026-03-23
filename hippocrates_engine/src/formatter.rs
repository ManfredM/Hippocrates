use crate::domain::EngineError;
use crate::parser::{preprocess_indentation, HippocratesParser, Rule};
use pest::Parser;
use pest::iterators::Pair;

const INDENT: &str = "    ";

pub fn format_script(input: &str) -> Result<String, EngineError> {
    let normalized = normalize_line_endings(input);
    let normalized = insert_statement_newlines(&normalized);
    let processed = preprocess_indentation(&normalized);

    let mut pairs = match HippocratesParser::parse(Rule::file, &processed) {
        Ok(p) => p,
        Err(e) => {
            let (line, column) = match e.line_col {
                pest::error::LineColLocation::Pos((l, c)) => (l, c),
                pest::error::LineColLocation::Span((l, c), _) => (l, c),
            };
            return Err(EngineError { suggestion: None,
                message: format!("Parsing error: {}", e),
                line,
                column,
            });
        }
    };

    let root = pairs.next().ok_or_else(|| EngineError { suggestion: None,
        message: "Parsing error: empty file".to_string(),
        line: 0,
        column: 0,
    })?;

    let mut out = String::new();
    format_file(root, &processed, 0, &mut out);

    if !out.ends_with('\n') {
        out.push('\n');
    }

    Ok(out)
}

fn normalize_line_endings(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push('\n');
        } else {
            out.push(ch);
        }
    }

    out
}

fn insert_statement_newlines(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut in_comment = false;
    let mut prev = '\0';
    let mut at_line_start = true;
    let mut indent_buf = String::new();

    while let Some(ch) = chars.next() {
        if at_line_start {
            if ch == ' ' || ch == '\t' {
                indent_buf.push(ch);
                out.push(ch);
                continue;
            }
            at_line_start = false;
        }

        if in_comment {
            out.push(ch);
            if ch == '*' && chars.peek() == Some(&')') {
                out.push(')');
                chars.next();
                in_comment = false;
            }
            continue;
        }

        if in_string {
            out.push(ch);
            if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '(' && chars.peek() == Some(&'*') {
            out.push(ch);
            out.push('*');
            chars.next();
            in_comment = true;
            continue;
        }

        if ch == '"' {
            out.push(ch);
            in_string = true;
            continue;
        }

        if ch == '.' {
            let next = chars.peek().copied();
            if prev == '.' || next == Some('.') {
                out.push(ch);
                prev = ch;
                continue;
            }

            if prev.is_ascii_digit() && next.map(|n| n.is_ascii_digit()).unwrap_or(false) {
                out.push(ch);
                prev = ch;
                continue;
            }

            out.push(ch);

            let mut look = chars.clone();
            let mut saw_newline = false;
            while let Some(n) = look.next() {
                if n == '\n' {
                    saw_newline = true;
                    break;
                }
                if n.is_whitespace() {
                    continue;
                }
                break;
            }

            if !saw_newline {
                while matches!(chars.peek(), Some(' ' | '\t')) {
                    chars.next();
                }
                out.push('\n');
                if !indent_buf.is_empty() {
                    out.push_str(&indent_buf);
                }
                prev = '\n';
                at_line_start = false;
                continue;
            }

            prev = ch;
            continue;
        }

        out.push(ch);
        if ch == '\n' {
            at_line_start = true;
            indent_buf.clear();
        }
        prev = ch;
    }

    out
}

fn format_file(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let mut first = true;
    for child in pair.into_inner() {
        if child.as_rule() != Rule::definition {
            continue;
        }
        if !first {
            out.push('\n');
        }
        format_definition(child, source, indent, out);
        first = false;
    }
}

fn format_definition(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let inner = pair.into_inner().next();
    if let Some(def_pair) = inner {
        match def_pair.as_rule() {
            Rule::unit_definition => format_unit_definition(def_pair, source, indent, out),
            Rule::value_definition => format_value_definition(def_pair, source, indent, out),
            Rule::period_definition => format_period_definition(def_pair, source, indent, out),
            Rule::plan_definition => format_plan_definition(def_pair, source, indent, out),
            Rule::drug_definition => format_drug_definition(def_pair, source, indent, out),
            Rule::addressee_definition => format_addressee_definition(def_pair, source, indent, out),
            Rule::context_definition => format_context_definition(def_pair, source, indent, out),
            _ => {
                write_line(out, indent, clean_line(def_pair.as_str()));
            }
        }
    }
}

fn format_unit_definition(pair: Pair<Rule>, _source: &str, indent: usize, out: &mut String) {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .map(|p| clean_line(p.as_str()))
        .unwrap_or_default();

    write_line(out, indent, format!("{} is a unit:", name));

    for prop in inner {
        if prop.as_rule() == Rule::unit_property {
            write_line(out, indent + 1, clean_line(prop.as_str()));
        }
    }
}

fn format_value_definition(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .map(|p| clean_line(p.as_str()))
        .unwrap_or_default();
    let value_type = inner
        .next()
        .map(|p| clean_line(p.as_str()))
        .unwrap_or_else(|| "a value".to_string());

    let props: Vec<Pair<Rule>> = inner.collect();
    if props.is_empty() {
        write_line(out, indent, format!("{} is {}.", name, value_type));
        return;
    }

    write_line(out, indent, format!("{} is {}:", name, value_type));
    for prop in props {
        if prop.as_rule() == Rule::value_property {
            format_value_property(prop, source, indent + 1, out);
        }
    }
}

fn format_value_property(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let inner = pair.into_inner().next();
    if let Some(prop) = inner {
        match prop.as_rule() {
            Rule::valid_values_prop => format_valid_values_prop(prop, source, indent, out),
            Rule::timeframe_prop => format_timeframe_prop(prop, source, indent, out),
            Rule::meaning_prop => format_meaning_prop(prop, source, indent, out),
            Rule::question_prop => format_block_property(prop, source, indent, out, "question:"),
            Rule::calculation_prop => format_block_property(prop, source, indent, out, "calculation:"),
            Rule::reuse_prop => format_reuse_prop(prop, indent, out),
            Rule::documentation_prop => format_documentation_prop(prop, indent, out),
            Rule::unit_ref_prop => write_line(out, indent, clean_line(prop.as_str())),
            Rule::inheritance_prop => format_inheritance_prop(prop, source, indent, out),
            Rule::generic_property => format_generic_property(prop, indent, out),
            _ => write_line(out, indent, clean_line(prop.as_str())),
        }
    }
}

fn format_valid_values_prop(pair: Pair<Rule>, _source: &str, indent: usize, out: &mut String) {
    write_line(out, indent, "valid values:".to_string());

    for line in pair.into_inner() {
        if line.as_rule() == Rule::valid_values_line {
            write_line(out, indent + 1, clean_line(line.as_str()));
        } else if line.as_rule() == Rule::valid_values_block {
            for child in line.into_inner() {
                if child.as_rule() == Rule::valid_values_line {
                    write_line(out, indent + 1, clean_line(child.as_str()));
                }
            }
        }
    }
}

fn format_timeframe_prop(pair: Pair<Rule>, _source: &str, indent: usize, out: &mut String) {
    write_line(out, indent, "timeframe:".to_string());
    for line in pair.into_inner() {
        if line.as_rule() == Rule::timeframe_line {
            write_line(out, indent + 1, clean_line(line.as_str()));
        }
    }
}

fn format_meaning_prop(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let mut target = None;
    let mut items = Vec::new();

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::identifier => target = Some(clean_line(child.as_str())),
            _ => items.push(child),
        }
    }

    let header = match target {
        Some(t) => format!("meaning of {}:", t),
        None => "meaning of <value>:".to_string(),
    };
    write_line(out, indent, header);

    for item in items {
        match item.as_rule() {
            Rule::valid_meanings_prop => format_valid_meanings_prop(item, indent + 1, out),
            Rule::meaning_assess_block => format_meaning_assess_block(item, source, indent + 1, out),
            Rule::assessment_case => format_assessment_case(item, source, indent + 1, out),
            _ => {}
        }
    }
}

fn format_valid_meanings_prop(pair: Pair<Rule>, indent: usize, out: &mut String) {
    write_line(out, indent, "valid meanings:".to_string());
    for line in pair.into_inner() {
        if line.as_rule() == Rule::valid_meanings_line {
            write_line(out, indent + 1, clean_line(line.as_str()));
        }
    }
}

fn format_meaning_assess_block(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let mut target = None;
    let mut cases = Vec::new();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::identifier => target = Some(clean_line(child.as_str())),
            Rule::assessment_case => cases.push(child),
            _ => {}
        }
    }

    let header = match target {
        Some(t) => format!("assess meaning of {}:", t),
        None => "assess meaning of <value>:".to_string(),
    };
    write_line(out, indent, header);

    for case in cases {
        format_assessment_case(case, source, indent + 1, out);
    }
}

fn format_block_property(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String, label: &str) {
    write_line(out, indent, label.to_string());
    for child in pair.into_inner() {
        if child.as_rule() == Rule::block_body {
            format_block_body(child, source, indent + 1, out);
        } else if child.as_rule() == Rule::statement {
            format_statement(child, source, indent + 1, out);
        }
    }
}

fn format_reuse_prop(pair: Pair<Rule>, indent: usize, out: &mut String) {
    write_line(out, indent, "reuse:".to_string());
    for stmt in pair.into_inner() {
        if stmt.as_rule() == Rule::reuse_stmt {
            write_line(out, indent + 1, clean_line(stmt.as_str()));
        }
    }
}

fn format_documentation_prop(pair: Pair<Rule>, indent: usize, out: &mut String) {
    let mut doc = None;
    for child in pair.into_inner() {
        if child.as_rule() == Rule::string_literal {
            doc = Some(clean_line(child.as_str()));
            break;
        }
    }

    write_line(out, indent, "documentation:".to_string());
    let line = match doc {
        Some(text) => format!("english: {}.", text),
        None => "english:".to_string(),
    };
    write_line(out, indent + 1, line);
}

fn format_inheritance_prop(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let mut inner = pair.into_inner();
    let base = inner
        .next()
        .map(|p| clean_line(p.as_str()))
        .unwrap_or_default();

    let mut overrides = Vec::new();
    for p in inner {
        if p.as_rule() == Rule::value_property {
            overrides.push(p);
        }
    }

    if overrides.is_empty() {
        write_line(out, indent, format!("definition is the same as for {}.", base));
        return;
    }

    write_line(
        out,
        indent,
        format!("definition is the same as for {} except:", base),
    );

    for prop in overrides {
        format_value_property(prop, source, indent + 1, out);
    }
}

fn format_generic_property(pair: Pair<Rule>, indent: usize, out: &mut String) {
    let raw = strip_markers(pair.as_str());
    let mut lines = raw.lines();
    if let Some(first) = lines.next() {
        write_line(out, indent, first.trim().to_string());
    }
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        write_line(out, indent + 1, trimmed.to_string());
    }
}

fn format_period_definition(pair: Pair<Rule>, _source: &str, indent: usize, out: &mut String) {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .map(|p| clean_line(p.as_str()))
        .unwrap_or_default();

    write_line(out, indent, format!("{} is a period:", name));

    for prop in inner {
        if prop.as_rule() != Rule::period_property {
            continue;
        }
        let raw = strip_markers(prop.as_str());
        let header = first_non_empty_line(&raw);
        if header.starts_with("timeframe") {
            write_line(out, indent + 1, "timeframe:".to_string());
            for line in prop.into_inner() {
                if line.as_rule() == Rule::timeframe_line {
                    write_line(out, indent + 2, clean_line(line.as_str()));
                }
            }
        } else if header.starts_with("customization") {
            write_line(out, indent + 1, "customization:".to_string());
            let mut lines = raw.lines().skip(1);
            for line in lines.by_ref() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                write_line(out, indent + 2, trimmed.to_string());
            }
        } else {
            write_line(out, indent + 1, clean_line(prop.as_str()));
        }
    }
}

fn format_plan_definition(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .map(|p| clean_line(p.as_str()))
        .unwrap_or_default();

    write_line(out, indent, format!("{} is a plan:", name));
    for block in inner {
        if block.as_rule() == Rule::plan_block {
            format_plan_block(block, source, indent + 1, out);
        }
    }
}

fn format_plan_block(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let inner = pair.into_inner().next();
    if let Some(block) = inner {
        let raw = strip_markers(block.as_str());
        let header = first_non_empty_line(&raw);
        write_line(out, indent, ensure_block_header(header));

        for child in block.into_inner() {
            if child.as_rule() == Rule::block_body {
                format_block_body(child, source, indent + 1, out);
            }
        }
    }
}

fn format_drug_definition(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .map(|p| clean_line(p.as_str()))
        .unwrap_or_default();

    write_line(out, indent, format!("{} is a drug:", name));

    for block in inner {
        match block.as_rule() {
            Rule::ingredients_block => format_simple_block(block, indent + 1, "ingredients:", out),
            Rule::dosage_block => format_simple_block(block, indent + 1, "dosage safety:", out),
            Rule::admin_block => format_simple_block(block, indent + 1, "administration:", out),
            Rule::interaction_block => format_interaction_block(block, source, indent + 1, out),
            _ => {}
        }
    }
}

fn format_addressee_definition(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let raw = strip_markers(pair.as_str());
    let header = first_non_empty_line(&raw);
    write_line(out, indent, ensure_block_header(header));

    for prop in pair.into_inner() {
        match prop.as_rule() {
            Rule::contact_info_prop => format_simple_block(prop, indent + 1, "contact information:", out),
            Rule::grouped_addressees_prop => write_line(out, indent + 1, clean_line(prop.as_str())),
            Rule::contact_logic_prop => format_simple_block(prop, indent + 1, "order of contacting:", out),
            Rule::after_consent_prop => format_after_consent(prop, source, indent + 1, out),
            _ => {}
        }
    }
}

fn format_context_definition(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    write_line(out, indent, "context:".to_string());
    for item in pair.into_inner() {
        if item.as_rule() == Rule::context_item {
            format_context_item(item, source, indent + 1, out);
        }
    }
}

fn format_context_item(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let mut has_case = None;
    for child in pair.clone().into_inner() {
        if child.as_rule() == Rule::assessment_case {
            has_case = Some(child);
            break;
        }
    }

    if let Some(case) = has_case {
        write_line(out, indent, "value filter:".to_string());
        format_assessment_case(case, source, indent + 1, out);
        return;
    }

    write_line(out, indent, clean_line(pair.as_str()));
}

fn format_after_consent(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    write_line(out, indent, "after consent has been rejected:".to_string());
    for child in pair.into_inner() {
        if child.as_rule() == Rule::block {
            for stmt in child.into_inner() {
                if stmt.as_rule() == Rule::statement {
                    format_statement(stmt, source, indent + 1, out);
                }
            }
        }
    }
}

fn format_interaction_block(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    write_line(out, indent, "interactions:".to_string());
    for child in pair.into_inner() {
        if child.as_rule() != Rule::interaction_rule {
            continue;
        }
        let raw = strip_markers(child.as_str());
        let header = first_non_empty_line(&raw);
        write_line(out, indent + 1, ensure_block_header(header));
        for block in child.into_inner() {
            if block.as_rule() == Rule::block {
                for stmt in block.into_inner() {
                    if stmt.as_rule() == Rule::statement {
                        format_statement(stmt, source, indent + 2, out);
                    }
                }
            }
        }
    }
}

fn format_simple_block(pair: Pair<Rule>, indent: usize, header: &str, out: &mut String) {
    write_line(out, indent, header.to_string());
    for line in pair.into_inner() {
        if line.as_rule() == Rule::ingredient
            || line.as_rule() == Rule::dosage_rule
            || line.as_rule() == Rule::admin_rule
            || line.as_rule() == Rule::contact_detail
            || line.as_rule() == Rule::contact_logic
        {
            write_line(out, indent + 1, clean_line(line.as_str()));
        }
    }
}

fn format_block_body(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    for stmt in pair.into_inner() {
        if stmt.as_rule() == Rule::statement {
            format_statement(stmt, source, indent, out);
        }
    }
}

fn format_statement(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let inner = pair.into_inner().next();
    if inner.is_none() {
        return;
    }

    let inner = inner.unwrap();
    match inner.as_rule() {
        Rule::assignment
        | Rule::meaning_assignment
        | Rule::constraint
        | Rule::documentation_prop
        | Rule::simple_command
        | Rule::start_period => {
            write_line(out, indent, clean_line(inner.as_str()));
        }
        Rule::action => format_action(inner, source, indent, out),
        Rule::conditional => format_conditional(inner, source, indent, out),
        Rule::context_block => format_context_block(inner, source, indent, out),
        Rule::timeframe_block => format_timeframe_block(inner, source, indent, out),
        Rule::NEWLINE => {}
        _ => {
            write_line(out, indent, clean_line(inner.as_str()));
        }
    }
}

fn format_action(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let inner = pair.into_inner().next();
    if inner.is_none() {
        return;
    }
    let inner = inner.unwrap();
    match inner.as_rule() {
        Rule::message_action => {
            if let Some(msg) = inner.into_inner().next() {
                format_message_action(msg, indent, out);
            }
        }
        Rule::information_message | Rule::warning_message | Rule::urgent_warning_message => {
            format_message_action(inner, indent, out)
        }
        Rule::ask_question => format_ask_question(inner, source, indent, out),
        Rule::listen_for => format_listen_for(inner, source, indent, out),
        Rule::question_modifier => format_question_modifier(inner, source, indent, out),
        Rule::send_info => write_line(out, indent, clean_line(inner.as_str())),
        Rule::start_period => write_line(out, indent, clean_line(inner.as_str())),
        Rule::simple_command => write_line(out, indent, clean_line(inner.as_str())),
        _ => write_line(out, indent, clean_line(inner.as_str())),
    }
}

fn format_message_action(pair: Pair<Rule>, indent: usize, out: &mut String) {
    let raw = strip_markers(pair.as_str());
    let header = first_non_empty_line(&raw);

    let has_block = raw.lines().count() > 1;
    if has_block {
        write_line(out, indent, ensure_block_header(header));
        for line in raw.lines().skip(1) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            write_line(out, indent + 1, trimmed.to_string());
        }
    } else {
        write_line(out, indent, header);
    }
}

fn format_ask_question(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let raw = strip_markers(pair.as_str());
    let header = first_non_empty_line(&raw);

    let mut block_body = None;
    for child in pair.into_inner() {
        if child.as_rule() == Rule::block_body {
            block_body = Some(child);
            break;
        }
    }

    if let Some(body) = block_body {
        write_line(out, indent, ensure_block_header(header));
        format_block_body(body, source, indent + 1, out);
    } else {
        write_line(out, indent, header);
    }
}

fn format_listen_for(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let raw = strip_markers(pair.as_str());
    let header = first_non_empty_line(&raw);
    write_line(out, indent, ensure_block_header(header));

    for child in pair.into_inner() {
        if child.as_rule() == Rule::block_body {
            format_block_body(child, source, indent + 1, out);
        }
    }
}

fn format_question_modifier(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let raw = strip_markers(pair.as_str());
    let header = first_non_empty_line(&raw);

    let mut block_body = None;
    let mut vas_block = None;
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::block_body => block_body = Some(child),
            Rule::vas_block => vas_block = Some(child),
            _ => {}
        }
    }

    if let Some(body) = block_body {
        write_line(out, indent, ensure_block_header(header));
        format_block_body(body, source, indent + 1, out);
        return;
    }

    if let Some(vas) = vas_block {
        write_line(out, indent, ensure_block_header(header));
        for item in vas.into_inner() {
            write_line(out, indent + 1, clean_line(item.as_str()));
        }
        return;
    }

    write_line(out, indent, header);
}

fn format_conditional(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let raw = strip_markers(pair.as_str());
    let header = first_non_empty_line(&raw);
    write_line(out, indent, ensure_block_header(header));

    for case in pair.into_inner() {
        if case.as_rule() == Rule::assessment_case {
            format_assessment_case(case, source, indent + 1, out);
        }
    }
}

fn format_assessment_case(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let mut selector_text = None;
    let mut block_body = None;

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::selector_list => selector_text = Some(clean_line(child.as_str())),
            Rule::block_body => block_body = Some(child),
            _ => {}
        }
    }

    let header = match selector_text {
        Some(text) => ensure_block_header(text),
        None => "".to_string(),
    };
    write_line(out, indent, header);

    if let Some(body) = block_body {
        format_block_body(body, source, indent + 1, out);
    }
}

fn format_context_block(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let raw = strip_markers(pair.as_str());
    let header = first_non_empty_line(&raw);
    write_line(out, indent, ensure_block_header(header));

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::context_item => format_context_item(child, source, indent + 1, out),
            Rule::statement => format_statement(child, source, indent + 1, out),
            _ => {}
        }
    }
}

fn format_timeframe_block(pair: Pair<Rule>, source: &str, indent: usize, out: &mut String) {
    let raw = strip_markers(pair.as_str());
    let header = first_non_empty_line(&raw);
    write_line(out, indent, ensure_block_header(header));

    for child in pair.into_inner() {
        if child.as_rule() == Rule::statement {
            format_statement(child, source, indent + 1, out);
        }
    }
}

fn write_line(out: &mut String, indent: usize, text: String) {
    if text.trim().is_empty() {
        return;
    }
    for _ in 0..indent {
        out.push_str(INDENT);
    }
    out.push_str(text.trim());
    out.push('\n');
}

fn strip_markers(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '《' && *c != '》')
        .collect()
}

fn clean_line(s: &str) -> String {
    strip_markers(s).trim().to_string()
}

fn first_non_empty_line(s: &str) -> String {
    for line in s.lines() {
        let cleaned = line.trim();
        if !cleaned.is_empty() {
            return cleaned.to_string();
        }
    }
    String::new()
}

fn ensure_block_header(header: String) -> String {
    let trimmed = header.trim_end();
    if trimmed.ends_with(':') {
        trimmed.to_string()
    } else {
        format!("{}:", trimmed)
    }
}
