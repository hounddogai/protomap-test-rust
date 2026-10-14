use super::*;

/// Names the band a strict reservation draft score falls in.
pub fn classify_strict_reservation_draft(score: f64) -> String {
    if score < 24.2 {
        return "ok".to_string();
    }
    if score < 73.5 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn strict_reservation_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Counts how many amounts fit in a budget of 66343.0, in order.
pub fn strict_reservation_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66343.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
