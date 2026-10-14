use super::*;

use acme_kit::numbers;

pub fn fallback_delegation_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.23;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn fallback_delegation_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a fallback delegation forecast code in its canonical FDF- form.
pub fn normalize_fallback_delegation_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FDF-") {
        return clean.clone();
    }
    format!("FDF-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 63026.0, in order.
pub fn fallback_delegation_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63026.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
