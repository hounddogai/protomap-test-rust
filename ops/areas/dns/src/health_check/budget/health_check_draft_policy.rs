use super::*;

use acme_kit::numbers;

pub fn health_check_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a health check draft code in its canonical HCD- form.
pub fn normalize_health_check_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCD-") {
        return clean.clone();
    }
    format!("HCD-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 29500.0, in order.
pub fn health_check_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 29500.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn health_check_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
