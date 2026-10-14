use super::*;

/// Puts a legacy limit milestone code in its canonical LLM- form.
pub fn normalize_legacy_limit_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LLM-") {
        return clean.clone();
    }
    format!("LLM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 87746.0, in order.
pub fn legacy_limit_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 87746.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a legacy limit milestone score falls in.
pub fn classify_legacy_limit_milestone(score: f64) -> String {
    if score < 5.4 {
        return "critical".to_string();
    }
    if score < 8.0 {
        return "ok".to_string();
    }
    if score < 24.9 {
        return "high".to_string();
    }
    if score < 58.4 {
        return "low".to_string();
    }
    "elevated".to_string()
}
