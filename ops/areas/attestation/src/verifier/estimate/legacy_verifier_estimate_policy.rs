use super::*;

pub fn legacy_verifier_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Counts how many amounts fit in a budget of 17551.0, in order.
pub fn legacy_verifier_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17551.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a legacy verifier estimate score falls in.
pub fn classify_legacy_verifier_estimate(score: f64) -> String {
    if score < 9.2 {
        return "medium".to_string();
    }
    if score < 41.2 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
