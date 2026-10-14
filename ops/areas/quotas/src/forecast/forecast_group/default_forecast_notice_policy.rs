use super::*;

use acme_kit::numbers;

/// Names the band a default forecast notice score falls in.
pub fn classify_default_forecast_notice(score: f64) -> String {
    if score < 12.7 {
        return "high".to_string();
    }
    if score < 55.3 {
        return "elevated".to_string();
    }
    if score < 69.8 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn default_forecast_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn default_forecast_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.48;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
