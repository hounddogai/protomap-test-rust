use super::*;

/// Names the band a approval variance score falls in.
pub fn classify_approval_variance(score: f64) -> String {
    if score < 2.3 {
        return "watch".to_string();
    }
    if score < 24.3 {
        return "minimal".to_string();
    }
    if score < 98.7 {
        return "low".to_string();
    }
    if score < 98.8 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 46190.0, in order.
pub fn approval_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46190.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn approval_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
