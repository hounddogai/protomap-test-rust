use super::*;

use acme_kit::numbers;

pub fn token_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

pub fn token_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a token preference score falls in.
pub fn classify_token_preference(score: f64) -> String {
    if score < 69.3 {
        return "watch".to_string();
    }
    if score < 70.5 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn token_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
