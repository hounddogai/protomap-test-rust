use super::*;

use acme_kit::numbers;

pub fn primary_rollout_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Puts a primary rollout estimate code in its canonical PRE- form.
pub fn normalize_primary_rollout_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRE-") {
        return clean.clone();
    }
    format!("PRE-{}", clean.replace(" ", "-"))
}

pub fn primary_rollout_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.57;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 93617.0, in order.
pub fn primary_rollout_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93617.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
