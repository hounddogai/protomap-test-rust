use super::*;

use acme_kit::numbers;

pub fn capped_sink_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn capped_sink_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Puts a capped sink adjustment code in its canonical CSA- form.
pub fn normalize_capped_sink_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CSA-") {
        return clean.clone();
    }
    format!("CSA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 88540.0, in order.
pub fn capped_sink_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 88540.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
