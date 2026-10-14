use super::*;

use acme_kit::numbers;

/// Puts a batched region pin window code in its canonical BRPW- form.
pub fn normalize_batched_region_pin_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BRPW-") {
        return clean.clone();
    }
    format!("BRPW-{}", clean.replace(" ", "-"))
}

pub fn batched_region_pin_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 84326.0, in order.
pub fn batched_region_pin_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84326.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched region pin window score falls in.
pub fn classify_batched_region_pin_window(score: f64) -> String {
    if score < 24.0 {
        return "minimal".to_string();
    }
    if score < 80.3 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
