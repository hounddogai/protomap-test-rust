use super::*;

use acme_kit::numbers;

pub fn regional_probe_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a regional probe forecast score falls in.
pub fn classify_regional_probe_forecast(score: f64) -> String {
    if score < 8.3 {
        return "high".to_string();
    }
    if score < 48.0 {
        return "minimal".to_string();
    }
    if score < 74.9 {
        return "critical".to_string();
    }
    if score < 94.5 {
        return "ok".to_string();
    }
    "watch".to_string()
}
