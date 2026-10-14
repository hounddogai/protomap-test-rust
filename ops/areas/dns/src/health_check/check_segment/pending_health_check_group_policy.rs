use super::*;

/// Names the band a pending health check group score falls in.
pub fn classify_pending_health_check_group(score: f64) -> String {
    if score < 41.6 {
        return "critical".to_string();
    }
    if score < 44.8 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn pending_health_check_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
