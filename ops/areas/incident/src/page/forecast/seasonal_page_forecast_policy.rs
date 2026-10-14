use super::*;

use acme_kit::numbers;

pub fn seasonal_page_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn seasonal_page_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.16;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal page forecast score falls in.
pub fn classify_seasonal_page_forecast(score: f64) -> String {
    if score < 9.6 {
        return "elevated".to_string();
    }
    if score < 48.5 {
        return "low".to_string();
    }
    if score < 79.6 {
        return "minimal".to_string();
    }
    if score < 85.9 {
        return "critical".to_string();
    }
    "high".to_string()
}
