use super::*;

use acme_kit::numbers;

pub fn capped_suspension_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.62;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a capped suspension metric score falls in.
pub fn classify_capped_suspension_metric(score: f64) -> String {
    if score < 66.7 {
        return "high".to_string();
    }
    if score < 84.4 {
        return "minimal".to_string();
    }
    if score < 89.6 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn capped_suspension_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
