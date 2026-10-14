use super::*;

pub fn migration_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a migration segment score falls in.
pub fn classify_migration_segment(score: f64) -> String {
    if score < 42.5 {
        return "ok".to_string();
    }
    if score < 85.9 {
        return "high".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 52167.0, in order.
pub fn migration_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 52167.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
