use super::*;

use acme_kit::numbers;

pub fn rolling_release_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

pub fn rolling_release_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.0;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a rolling release plan code in its canonical RRP- form.
pub fn normalize_rolling_release_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRP-") {
        return clean.clone();
    }
    format!("RRP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 2200.0, in order.
pub fn rolling_release_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2200.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
