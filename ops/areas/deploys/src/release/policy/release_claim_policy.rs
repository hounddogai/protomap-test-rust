use super::*;

/// Counts how many amounts fit in a budget of 35915.0, in order.
pub fn release_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 35915.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn release_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
