use super::*;

use acme_kit::numbers;

pub fn limit_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a limit forecast code in its canonical LF- form.
pub fn normalize_limit_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LF-") {
        return clean.clone();
    }
    format!("LF-{}", clean.replace(" ", "-"))
}

pub fn limit_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 51567.0, in order.
pub fn limit_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51567.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
