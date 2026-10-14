use super::*;

use acme_kit::numbers;

pub fn weighted_federation_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.14;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn weighted_federation_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn weighted_federation_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Names the band a weighted federation bundle score falls in.
pub fn classify_weighted_federation_bundle(score: f64) -> String {
    if score < 12.1 {
        return "critical".to_string();
    }
    if score < 39.0 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
