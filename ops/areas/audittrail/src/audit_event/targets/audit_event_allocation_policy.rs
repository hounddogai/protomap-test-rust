use super::*;

/// Names the band a audit event allocation score falls in.
pub fn classify_audit_event_allocation(score: f64) -> String {
    if score < 16.8 {
        return "watch".to_string();
    }
    if score < 63.6 {
        return "medium".to_string();
    }
    if score < 67.9 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

pub fn audit_event_allocation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
