use super::*;

use acme_kit::numbers;

pub fn overage_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.9;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a overage metric score falls in.
pub fn classify_overage_metric(score: f64) -> String {
    if score < 51.4 {
        return "critical".to_string();
    }
    if score < 68.9 {
        return "watch".to_string();
    }
    if score < 93.9 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
