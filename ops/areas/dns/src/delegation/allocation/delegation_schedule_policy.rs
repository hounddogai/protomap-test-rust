use super::*;

pub fn delegation_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a delegation schedule score falls in.
pub fn classify_delegation_schedule(score: f64) -> String {
    if score < 10.1 {
        return "elevated".to_string();
    }
    if score < 25.3 {
        return "low".to_string();
    }
    if score < 52.0 {
        return "medium".to_string();
    }
    if score < 85.2 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

/// Puts a delegation schedule code in its canonical DS- form.
pub fn normalize_delegation_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DS-") {
        return clean.clone();
    }
    format!("DS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 88035.0, in order.
pub fn delegation_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 88035.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
