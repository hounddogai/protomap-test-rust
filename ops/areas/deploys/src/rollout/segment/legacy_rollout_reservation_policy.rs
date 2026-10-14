use super::*;

/// Counts how many amounts fit in a budget of 5774.0, in order.
pub fn legacy_rollout_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 5774.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn legacy_rollout_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Puts a legacy rollout reservation code in its canonical LRR- form.
pub fn normalize_legacy_rollout_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRR-") {
        return clean.clone();
    }
    format!("LRR-{}", clean.replace(" ", "-"))
}
