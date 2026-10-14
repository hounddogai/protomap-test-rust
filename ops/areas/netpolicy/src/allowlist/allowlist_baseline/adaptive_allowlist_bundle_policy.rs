use super::*;

use acme_kit::numbers;

pub fn adaptive_allowlist_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 37839.0, in order.
pub fn adaptive_allowlist_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 37839.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a adaptive allowlist bundle code in its canonical AAB- form.
pub fn normalize_adaptive_allowlist_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AAB-") {
        return clean.clone();
    }
    format!("AAB-{}", clean.replace(" ", "-"))
}

pub fn adaptive_allowlist_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
