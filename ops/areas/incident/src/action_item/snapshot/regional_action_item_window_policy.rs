use super::*;

use acme_kit::numbers;

pub fn regional_action_item_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a regional action item window code in its canonical RAIW- form.
pub fn normalize_regional_action_item_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RAIW-") {
        return clean.clone();
    }
    format!("RAIW-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 13254.0, in order.
pub fn regional_action_item_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 13254.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn regional_action_item_window_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
