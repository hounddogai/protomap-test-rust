use super::*;

use acme_kit::numbers;

pub fn tiered_trust_domain_checkpoint_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.5;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a tiered trust domain checkpoint code in its canonical TTDC- form.
pub fn normalize_tiered_trust_domain_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TTDC-") {
        return clean.clone();
    }
    format!("TTDC-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 90431.0, in order.
pub fn tiered_trust_domain_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90431.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn tiered_trust_domain_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
