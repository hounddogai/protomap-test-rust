use super::*;

pub fn fallback_binding_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Counts how many amounts fit in a budget of 29675.0, in order.
pub fn fallback_binding_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 29675.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
