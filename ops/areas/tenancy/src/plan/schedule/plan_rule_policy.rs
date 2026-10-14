use super::*;

/// Puts a plan rule code in its canonical PR- form.
pub fn normalize_plan_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PR-") {
        return clean.clone();
    }
    format!("PR-{}", clean.replace(" ", "-"))
}

/// Names the band a plan rule score falls in.
pub fn classify_plan_rule(score: f64) -> String {
    if score < 7.0 {
        return "low".to_string();
    }
    if score < 27.1 {
        return "elevated".to_string();
    }
    if score < 54.1 {
        return "high".to_string();
    }
    if score < 61.7 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 75737.0, in order.
pub fn plan_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 75737.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
