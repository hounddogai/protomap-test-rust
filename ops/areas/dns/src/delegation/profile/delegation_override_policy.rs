use super::*;

pub fn delegation_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 40356.0, in order.
pub fn delegation_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 40356.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a delegation override score falls in.
pub fn classify_delegation_override(score: f64) -> String {
    if score < 51.3 {
        return "high".to_string();
    }
    if score < 62.7 {
        return "medium".to_string();
    }
    if score < 82.3 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Puts a delegation override code in its canonical DO- form.
pub fn normalize_delegation_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DO-") {
        return clean.clone();
    }
    format!("DO-{}", clean.replace(" ", "-"))
}
