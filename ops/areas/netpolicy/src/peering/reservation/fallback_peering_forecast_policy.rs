use super::*;

use acme_kit::numbers;

pub fn fallback_peering_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a fallback peering forecast code in its canonical FPF- form.
pub fn normalize_fallback_peering_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FPF-") {
        return clean.clone();
    }
    format!("FPF-{}", clean.replace(" ", "-"))
}

pub fn fallback_peering_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 41153.0, in order.
pub fn fallback_peering_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 41153.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
