use super::*;

use acme_kit::numbers;

pub fn measurement_baseline_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn measurement_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a measurement baseline score falls in.
pub fn classify_measurement_baseline(score: f64) -> String {
    if score < 2.7 {
        return "elevated".to_string();
    }
    if score < 15.7 {
        return "low".to_string();
    }
    if score < 51.2 {
        return "critical".to_string();
    }
    if score < 69.2 {
        return "medium".to_string();
    }
    "ok".to_string()
}
