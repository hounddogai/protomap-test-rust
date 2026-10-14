use super::*;

use acme_kit::numbers;

pub fn primary_overage_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.09;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a primary overage forecast score falls in.
pub fn classify_primary_overage_forecast(score: f64) -> String {
    if score < 53.2 {
        return "critical".to_string();
    }
    if score < 65.8 {
        return "ok".to_string();
    }
    if score < 78.2 {
        return "high".to_string();
    }
    if score < 97.6 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn primary_overage_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
