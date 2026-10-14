use super::*;

use acme_kit::numbers;

/// Names the band a pending approval digest score falls in.
pub fn classify_pending_approval_digest(score: f64) -> String {
    if score < 22.4 {
        return "elevated".to_string();
    }
    if score < 55.4 {
        return "minimal".to_string();
    }
    if score < 60.4 {
        return "high".to_string();
    }
    if score < 90.8 {
        return "medium".to_string();
    }
    "critical".to_string()
}

pub fn pending_approval_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

pub fn pending_approval_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
