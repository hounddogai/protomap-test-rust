use super::*;

use acme_kit::numbers;

pub fn suspension_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.63;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn suspension_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a suspension group score falls in.
pub fn classify_suspension_group(score: f64) -> String {
    if score < 11.1 {
        return "high".to_string();
    }
    if score < 81.2 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
