use super::*;

use acme_kit::numbers;

/// Puts a ttl policy variance code in its canonical TPV- form.
pub fn normalize_ttl_policy_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TPV-") {
        return clean.clone();
    }
    format!("TPV-{}", clean.replace(" ", "-"))
}

pub fn ttl_policy_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

pub fn ttl_policy_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 32322.0, in order.
pub fn ttl_policy_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 32322.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
