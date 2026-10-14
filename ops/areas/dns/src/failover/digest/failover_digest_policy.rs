use super::*;

pub fn failover_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a failover digest score falls in.
pub fn classify_failover_digest(score: f64) -> String {
    if score < 6.1 {
        return "watch".to_string();
    }
    if score < 20.5 {
        return "minimal".to_string();
    }
    if score < 44.5 {
        return "ok".to_string();
    }
    if score < 89.4 {
        return "medium".to_string();
    }
    "critical".to_string()
}
