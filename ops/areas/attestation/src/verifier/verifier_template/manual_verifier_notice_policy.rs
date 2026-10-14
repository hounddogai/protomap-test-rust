use super::*;

pub fn manual_verifier_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 31507.0, in order.
pub fn manual_verifier_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 31507.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
