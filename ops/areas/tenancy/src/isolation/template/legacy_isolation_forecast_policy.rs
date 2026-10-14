use super::*;

use acme_kit::numbers;

pub fn legacy_isolation_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a legacy isolation forecast score falls in.
pub fn classify_legacy_isolation_forecast(score: f64) -> String {
    if score < 44.3 {
        return "watch".to_string();
    }
    if score < 55.8 {
        return "critical".to_string();
    }
    if score < 71.6 {
        return "minimal".to_string();
    }
    if score < 84.0 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn legacy_isolation_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
