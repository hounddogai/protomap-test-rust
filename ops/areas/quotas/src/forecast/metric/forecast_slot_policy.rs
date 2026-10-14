use super::*;

use acme_kit::numbers;

pub fn forecast_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.35;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a forecast slot score falls in.
pub fn classify_forecast_slot(score: f64) -> String {
    if score < 5.5 {
        return "watch".to_string();
    }
    if score < 7.4 {
        return "critical".to_string();
    }
    if score < 32.1 {
        return "minimal".to_string();
    }
    "high".to_string()
}
