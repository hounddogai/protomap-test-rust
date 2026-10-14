use super::*;

/// Counts how many amounts fit in a budget of 94705.0, in order.
pub fn segment_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 94705.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn segment_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
