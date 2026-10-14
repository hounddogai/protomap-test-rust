use super::*;

use acme_kit::numbers;

pub fn federation_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a federation batch code in its canonical FB- form.
pub fn normalize_federation_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FB-") {
        return clean.clone();
    }
    format!("FB-{}", clean.replace(" ", "-"))
}

pub fn federation_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 72775.0, in order.
pub fn federation_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72775.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
