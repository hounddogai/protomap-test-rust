use super::*;

/// Puts a rollback bundle code in its canonical RB- form.
pub fn normalize_rollback_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RB-") {
        return clean.clone();
    }
    format!("RB-{}", clean.replace(" ", "-"))
}

/// Names the band a rollback bundle score falls in.
pub fn classify_rollback_bundle(score: f64) -> String {
    if score < 5.6 {
        return "medium".to_string();
    }
    if score < 52.8 {
        return "watch".to_string();
    }
    if score < 87.9 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn rollback_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 26974.0, in order.
pub fn rollback_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 26974.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
