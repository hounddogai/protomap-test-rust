use super::*;

/// Names the band a evidence quota score falls in.
pub fn classify_evidence_quota(score: f64) -> String {
    if score < 30.7 {
        return "critical".to_string();
    }
    if score < 31.0 {
        return "watch".to_string();
    }
    if score < 56.5 {
        return "ok".to_string();
    }
    if score < 70.5 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn evidence_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
