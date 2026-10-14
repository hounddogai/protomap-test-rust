use super::*;

pub fn manual_peering_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a manual peering entry score falls in.
pub fn classify_manual_peering_entry(score: f64) -> String {
    if score < 5.1 {
        return "low".to_string();
    }
    if score < 9.4 {
        return "medium".to_string();
    }
    if score < 85.8 {
        return "ok".to_string();
    }
    if score < 88.3 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
