use super::*;

use acme_kit::numbers;

pub fn forecast_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Names the band a forecast group score falls in.
pub fn classify_forecast_group(score: f64) -> String {
    if score < 72.6 {
        return "watch".to_string();
    }
    if score < 73.3 {
        return "ok".to_string();
    }
    if score < 81.4 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn forecast_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
