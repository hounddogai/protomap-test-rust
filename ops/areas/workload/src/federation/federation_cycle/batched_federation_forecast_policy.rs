use super::*;

use acme_kit::numbers;

/// Names the band a batched federation forecast score falls in.
pub fn classify_batched_federation_forecast(score: f64) -> String {
    if score < 30.8 {
        return "high".to_string();
    }
    if score < 38.3 {
        return "elevated".to_string();
    }
    if score < 42.5 {
        return "medium".to_string();
    }
    "critical".to_string()
}

pub fn batched_federation_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn batched_federation_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
