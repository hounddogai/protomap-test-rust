use super::*;

use acme_kit::numbers;

/// Puts a egress schedule code in its canonical ES- form.
pub fn normalize_egress_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ES-") {
        return clean.clone();
    }
    format!("ES-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 66602.0, in order.
pub fn egress_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66602.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn egress_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn egress_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.05;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
