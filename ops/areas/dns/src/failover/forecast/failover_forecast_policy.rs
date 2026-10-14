use super::*;

use acme_kit::numbers;

/// Names the band a failover forecast score falls in.
pub fn classify_failover_forecast(score: f64) -> String {
    if score < 32.0 {
        return "critical".to_string();
    }
    if score < 49.8 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn failover_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.85;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
