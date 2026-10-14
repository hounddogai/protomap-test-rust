use super::*;

pub fn dns_record_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a dns record variance score falls in.
pub fn classify_dns_record_variance(score: f64) -> String {
    if score < 26.3 {
        return "critical".to_string();
    }
    if score < 63.5 {
        return "medium".to_string();
    }
    if score < 80.8 {
        return "ok".to_string();
    }
    if score < 83.4 {
        return "watch".to_string();
    }
    "low".to_string()
}
