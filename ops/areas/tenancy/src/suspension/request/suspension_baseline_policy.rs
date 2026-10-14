use super::*;

use acme_kit::numbers;

/// Names the band a suspension baseline score falls in.
pub fn classify_suspension_baseline(score: f64) -> String {
    if score < 22.6 {
        return "minimal".to_string();
    }
    if score < 36.5 {
        return "medium".to_string();
    }
    if score < 60.5 {
        return "high".to_string();
    }
    "critical".to_string()
}

pub fn suspension_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.47;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
