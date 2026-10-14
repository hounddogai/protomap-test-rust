use super::*;

use acme_kit::numbers;

pub fn grant_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn grant_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a grant forecast score falls in.
pub fn classify_grant_forecast(score: f64) -> String {
    if score < 72.6 {
        return "medium".to_string();
    }
    if score < 89.2 {
        return "elevated".to_string();
    }
    if score < 90.3 {
        return "low".to_string();
    }
    if score < 94.5 {
        return "watch".to_string();
    }
    "critical".to_string()
}
