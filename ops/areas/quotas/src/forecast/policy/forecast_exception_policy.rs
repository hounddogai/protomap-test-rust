use super::*;

use acme_kit::numbers;

/// Names the band a forecast exception score falls in.
pub fn classify_forecast_exception(score: f64) -> String {
    if score < 47.2 {
        return "watch".to_string();
    }
    if score < 51.5 {
        return "medium".to_string();
    }
    if score < 92.8 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn forecast_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.39;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
