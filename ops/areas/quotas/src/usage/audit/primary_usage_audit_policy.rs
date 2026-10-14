use super::*;

pub fn primary_usage_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a primary usage audit score falls in.
pub fn classify_primary_usage_audit(score: f64) -> String {
    if score < 72.0 {
        return "medium".to_string();
    }
    if score < 99.5 {
        return "watch".to_string();
    }
    "ok".to_string()
}
