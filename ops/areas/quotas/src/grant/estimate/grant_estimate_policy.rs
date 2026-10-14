use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 66074.0, in order.
pub fn grant_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66074.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn grant_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

pub fn grant_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.07;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a grant estimate code in its canonical GE- form.
pub fn normalize_grant_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GE-") {
        return clean.clone();
    }
    format!("GE-{}", clean.replace(" ", "-"))
}
