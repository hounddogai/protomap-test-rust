use super::*;

use acme_kit::numbers;

/// Puts a global audit event schedule code in its canonical GAES- form.
pub fn normalize_global_audit_event_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GAES-") {
        return clean.clone();
    }
    format!("GAES-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 71087.0, in order.
pub fn global_audit_event_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 71087.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn global_audit_event_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

pub fn global_audit_event_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
