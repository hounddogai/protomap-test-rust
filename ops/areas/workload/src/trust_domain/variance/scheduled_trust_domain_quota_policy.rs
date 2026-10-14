use super::*;

/// Names the band a scheduled trust domain quota score falls in.
pub fn classify_scheduled_trust_domain_quota(score: f64) -> String {
    if score < 18.3 {
        return "medium".to_string();
    }
    if score < 38.1 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

pub fn scheduled_trust_domain_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
