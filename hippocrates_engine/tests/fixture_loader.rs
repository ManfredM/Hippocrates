#[allow(dead_code)]
pub enum ScenarioKind {
    Pass,
    Fail,
}

pub fn load_scenario(path: &str, name: &str, kind: ScenarioKind) -> String {
    let contents = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Failed to read {}: {}", path, err));

    let kind_str = match kind {
        ScenarioKind::Pass => "PASS",
        ScenarioKind::Fail => "FAIL",
    };
    let marker = format!("(* --- {}: {} --- *)", kind_str, name);

    let mut in_section = false;
    let mut lines = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("(* --- ") && trimmed.ends_with(" --- *)") {
            if in_section {
                break;
            }
            if trimmed == marker {
                in_section = true;
            }
            continue;
        }

        if in_section {
            lines.push(line);
        }
    }

    if !in_section {
        panic!(
            "Scenario not found: {} ({}) in {}",
            name,
            kind_str,
            path
        );
    }

    let body = lines.join("\n");
    body.trim().to_string()
}

pub fn list_scenarios(path: &str, kind: ScenarioKind) -> Vec<String> {
    let contents = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Failed to read {}: {}", path, err));

    let kind_str = match kind {
        ScenarioKind::Pass => "PASS",
        ScenarioKind::Fail => "FAIL",
    };

    let prefix = format!("(* --- {}: ", kind_str);
    let suffix = " --- *)";
    let mut scenarios = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&prefix) && trimmed.ends_with(suffix) {
            let name = &trimmed[prefix.len()..trimmed.len() - suffix.len()];
            scenarios.push(name.to_string());
        }
    }

    scenarios
}
