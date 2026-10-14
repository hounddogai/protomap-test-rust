use super::*;

use acme_kit::numbers;

pub fn peering_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a peering segment score falls in.
pub fn classify_peering_segment(score: f64) -> String {
    if score < 3.5 {
        return "critical".to_string();
    }
    if score < 41.7 {
        return "ok".to_string();
    }
    if score < 63.5 {
        return "watch".to_string();
    }
    if score < 68.8 {
        return "low".to_string();
    }
    "high".to_string()
}

pub fn peering_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
