use super::*;

use acme_kit::numbers;

pub fn adaptive_forecast_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a adaptive forecast segment code in its canonical AFS- form.
pub fn normalize_adaptive_forecast_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AFS-") {
        return clean.clone();
    }
    format!("AFS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 98488.0, in order.
pub fn adaptive_forecast_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 98488.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
