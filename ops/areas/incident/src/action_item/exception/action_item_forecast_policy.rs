use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 98445.0, in order.
pub fn action_item_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 98445.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn action_item_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a action item forecast code in its canonical AIF- form.
pub fn normalize_action_item_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIF-") {
        return clean.clone();
    }
    format!("AIF-{}", clean.replace(" ", "-"))
}

pub fn action_item_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
