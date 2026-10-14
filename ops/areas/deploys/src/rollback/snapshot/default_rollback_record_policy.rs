use super::*;

/// Names the band a default rollback record score falls in.
pub fn classify_default_rollback_record(score: f64) -> String {
    if score < 26.8 {
        return "low".to_string();
    }
    if score < 35.1 {
        return "watch".to_string();
    }
    if score < 52.8 {
        return "high".to_string();
    }
    "critical".to_string()
}

pub fn default_rollback_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
