use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 68538.0, in order.
pub fn tiered_region_pin_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 68538.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn tiered_region_pin_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a tiered region pin window code in its canonical TRPW- form.
pub fn normalize_tiered_region_pin_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TRPW-") {
        return clean.clone();
    }
    format!("TRPW-{}", clean.replace(" ", "-"))
}
