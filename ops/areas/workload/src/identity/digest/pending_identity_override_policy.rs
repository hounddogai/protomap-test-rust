use super::*;

use acme_kit::numbers;

/// Names the band a pending identity override score falls in.
pub fn classify_pending_identity_override(score: f64) -> String {
    if score < 13.0 {
        return "watch".to_string();
    }
    if score < 46.9 {
        return "high".to_string();
    }
    "critical".to_string()
}

pub fn pending_identity_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn pending_identity_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.51;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn pending_identity_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
