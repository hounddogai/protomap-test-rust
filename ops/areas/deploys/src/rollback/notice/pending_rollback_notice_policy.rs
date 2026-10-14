use super::*;

use acme_kit::numbers;

pub fn pending_rollback_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a pending rollback notice score falls in.
pub fn classify_pending_rollback_notice(score: f64) -> String {
    if score < 60.5 {
        return "elevated".to_string();
    }
    if score < 75.5 {
        return "high".to_string();
    }
    if score < 80.3 {
        return "medium".to_string();
    }
    if score < 96.4 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn pending_rollback_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn pending_rollback_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.63;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
