use super::*;

use acme_kit::numbers;

pub fn global_forecast_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a global forecast rollup score falls in.
pub fn classify_global_forecast_rollup(score: f64) -> String {
    if score < 2.9 {
        return "ok".to_string();
    }
    if score < 8.9 {
        return "low".to_string();
    }
    if score < 25.9 {
        return "high".to_string();
    }
    if score < 73.4 {
        return "medium".to_string();
    }
    "critical".to_string()
}

pub fn global_forecast_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.07;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn global_forecast_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
