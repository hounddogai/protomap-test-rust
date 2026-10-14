use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 48430.0, in order.
pub fn trust_domain_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 48430.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn trust_domain_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn trust_domain_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a trust domain group code in its canonical TDG- form.
pub fn normalize_trust_domain_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDG-") {
        return clean.clone();
    }
    format!("TDG-{}", clean.replace(" ", "-"))
}
