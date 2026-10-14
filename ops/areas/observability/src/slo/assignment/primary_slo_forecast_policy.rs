use super::*;

use acme_kit::numbers;

/// Names the band a primary slo forecast score falls in.
pub fn classify_primary_slo_forecast(score: f64) -> String {
    if score < 17.8 {
        return "watch".to_string();
    }
    if score < 32.3 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn primary_slo_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.17;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
