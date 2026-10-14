use super::*;

use acme_kit::numbers;

pub fn scheduled_policy_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn scheduled_policy_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.8;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a scheduled policy digest score falls in.
pub fn classify_scheduled_policy_digest(score: f64) -> String {
    if score < 4.6 {
        return "high".to_string();
    }
    if score < 28.8 {
        return "critical".to_string();
    }
    "ok".to_string()
}
