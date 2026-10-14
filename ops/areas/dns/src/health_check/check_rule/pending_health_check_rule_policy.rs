use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 68511.0, in order.
pub fn pending_health_check_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 68511.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn pending_health_check_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

pub fn pending_health_check_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a pending health check rule code in its canonical PHCR- form.
pub fn normalize_pending_health_check_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PHCR-") {
        return clean.clone();
    }
    format!("PHCR-{}", clean.replace(" ", "-"))
}
