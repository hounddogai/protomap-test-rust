use super::*;

use acme_kit::numbers;

pub fn manual_ttl_policy_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn manual_ttl_policy_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 77759.0, in order.
pub fn manual_ttl_policy_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 77759.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a manual ttl policy override code in its canonical MTPO- form.
pub fn normalize_manual_ttl_policy_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MTPO-") {
        return clean.clone();
    }
    format!("MTPO-{}", clean.replace(" ", "-"))
}
