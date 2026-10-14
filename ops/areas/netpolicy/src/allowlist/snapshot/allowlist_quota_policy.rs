use super::*;

use acme_kit::numbers;

/// Names the band a allowlist quota score falls in.
pub fn classify_allowlist_quota(score: f64) -> String {
    if score < 19.0 {
        return "low".to_string();
    }
    if score < 35.8 {
        return "ok".to_string();
    }
    if score < 74.2 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn allowlist_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn allowlist_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
