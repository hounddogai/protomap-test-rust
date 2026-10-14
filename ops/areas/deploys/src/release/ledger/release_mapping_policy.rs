use super::*;

pub fn release_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Counts how many amounts fit in a budget of 74062.0, in order.
pub fn release_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74062.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a release mapping score falls in.
pub fn classify_release_mapping(score: f64) -> String {
    if score < 15.4 {
        return "elevated".to_string();
    }
    if score < 40.4 {
        return "low".to_string();
    }
    if score < 69.0 {
        return "minimal".to_string();
    }
    if score < 85.7 {
        return "critical".to_string();
    }
    "medium".to_string()
}
