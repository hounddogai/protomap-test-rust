use super::*;

use acme_kit::numbers;

pub fn release_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a release preference score falls in.
pub fn classify_release_preference(score: f64) -> String {
    if score < 3.0 {
        return "elevated".to_string();
    }
    if score < 31.3 {
        return "watch".to_string();
    }
    if score < 33.2 {
        return "medium".to_string();
    }
    if score < 42.5 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn release_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
