use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 57261.0, in order.
pub fn forecast_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 57261.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn forecast_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a forecast entry code in its canonical FE- form.
pub fn normalize_forecast_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FE-") {
        return clean.clone();
    }
    format!("FE-{}", clean.replace(" ", "-"))
}

pub fn forecast_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
