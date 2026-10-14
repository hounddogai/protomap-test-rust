use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 85993.0, in order.
pub fn limit_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85993.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a limit override code in its canonical LO- form.
pub fn normalize_limit_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LO-") {
        return clean.clone();
    }
    format!("LO-{}", clean.replace(" ", "-"))
}

pub fn limit_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.02;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn limit_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
