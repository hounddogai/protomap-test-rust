use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 57117.0, in order.
pub fn scheduled_trust_domain_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 57117.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn scheduled_trust_domain_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn scheduled_trust_domain_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a scheduled trust domain tier code in its canonical STDT- form.
pub fn normalize_scheduled_trust_domain_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("STDT-") {
        return clean.clone();
    }
    format!("STDT-{}", clean.replace(" ", "-"))
}
