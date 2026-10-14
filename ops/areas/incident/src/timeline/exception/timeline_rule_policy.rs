use super::*;

/// Names the band a timeline rule score falls in.
pub fn classify_timeline_rule(score: f64) -> String {
    if score < 52.0 {
        return "watch".to_string();
    }
    if score < 84.3 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn timeline_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
