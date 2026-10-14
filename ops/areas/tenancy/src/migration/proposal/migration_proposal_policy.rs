use super::*;

pub fn migration_proposal_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a migration proposal score falls in.
pub fn classify_migration_proposal(score: f64) -> String {
    if score < 42.3 {
        return "high".to_string();
    }
    if score < 52.8 {
        return "watch".to_string();
    }
    "critical".to_string()
}
