use super::*;

use acme_kit::numbers;

/// Puts a default identity forecast code in its canonical DIF- form.
pub fn normalize_default_identity_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DIF-") {
        return clean.clone();
    }
    format!("DIF-{}", clean.replace(" ", "-"))
}

pub fn default_identity_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn default_identity_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.14;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 15948.0, in order.
pub fn default_identity_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 15948.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
