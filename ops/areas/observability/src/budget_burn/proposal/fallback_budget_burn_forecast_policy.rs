use super::*;

use acme_kit::numbers;

/// Names the band a fallback budget burn forecast score falls in.
pub fn classify_fallback_budget_burn_forecast(score: f64) -> String {
    if score < 41.9 {
        return "ok".to_string();
    }
    if score < 55.6 {
        return "elevated".to_string();
    }
    if score < 63.8 {
        return "critical".to_string();
    }
    if score < 92.2 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn fallback_budget_burn_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.21;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
