use super::*;

use acme_kit::numbers;

/// Names the band a postmortem estimate score falls in.
pub fn classify_postmortem_estimate(score: f64) -> String {
    if score < 75.1 {
        return "elevated".to_string();
    }
    if score < 95.5 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn postmortem_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn postmortem_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.11;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
