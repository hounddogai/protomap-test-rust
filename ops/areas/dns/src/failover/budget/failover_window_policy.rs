use super::*;

use acme_kit::numbers;

/// Puts a failover window code in its canonical FW- form.
pub fn normalize_failover_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FW-") {
        return clean.clone();
    }
    format!("FW-{}", clean.replace(" ", "-"))
}

pub fn failover_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 82399.0, in order.
pub fn failover_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 82399.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn failover_window_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
