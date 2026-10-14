use super::*;

/// Names the band a freeze record score falls in.
pub fn classify_freeze_record(score: f64) -> String {
    if score < 3.4 {
        return "medium".to_string();
    }
    if score < 7.6 {
        return "watch".to_string();
    }
    if score < 24.3 {
        return "critical".to_string();
    }
    if score < 50.7 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn freeze_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
