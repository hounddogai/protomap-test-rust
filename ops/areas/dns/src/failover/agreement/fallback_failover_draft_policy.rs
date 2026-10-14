use super::*;

use acme_kit::numbers;

pub fn fallback_failover_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.76;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn fallback_failover_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a fallback failover draft score falls in.
pub fn classify_fallback_failover_draft(score: f64) -> String {
    if score < 7.1 {
        return "critical".to_string();
    }
    if score < 30.9 {
        return "ok".to_string();
    }
    if score < 67.6 {
        return "medium".to_string();
    }
    "watch".to_string()
}
