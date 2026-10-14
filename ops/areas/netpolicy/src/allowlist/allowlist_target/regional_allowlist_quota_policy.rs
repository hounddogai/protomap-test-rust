use super::*;

use acme_kit::numbers;

pub fn regional_allowlist_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

pub fn regional_allowlist_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.48;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a regional allowlist quota score falls in.
pub fn classify_regional_allowlist_quota(score: f64) -> String {
    if score < 12.1 {
        return "high".to_string();
    }
    if score < 54.3 {
        return "watch".to_string();
    }
    if score < 67.3 {
        return "medium".to_string();
    }
    if score < 90.3 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn regional_allowlist_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
