use super::*;

use acme_kit::numbers;

/// Names the band a forecast preference score falls in.
pub fn classify_forecast_preference(score: f64) -> String {
    if score < 21.3 {
        return "critical".to_string();
    }
    if score < 37.8 {
        return "low".to_string();
    }
    "high".to_string()
}

pub fn forecast_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
