use super::*;

use acme_kit::numbers;

/// Names the band a sink review score falls in.
pub fn classify_sink_review(score: f64) -> String {
    if score < 5.2 {
        return "high".to_string();
    }
    if score < 18.1 {
        return "critical".to_string();
    }
    if score < 37.9 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn sink_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.8;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
