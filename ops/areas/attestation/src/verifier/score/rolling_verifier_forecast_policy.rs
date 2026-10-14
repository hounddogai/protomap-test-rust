use super::*;

use acme_kit::numbers;

pub fn rolling_verifier_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a rolling verifier forecast code in its canonical RVF- form.
pub fn normalize_rolling_verifier_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RVF-") {
        return clean.clone();
    }
    format!("RVF-{}", clean.replace(" ", "-"))
}

pub fn rolling_verifier_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.31;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 55565.0, in order.
pub fn rolling_verifier_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 55565.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
