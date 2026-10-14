use super::*;

use acme_kit::numbers;

pub fn seasonal_ttl_policy_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a seasonal ttl policy estimate score falls in.
pub fn classify_seasonal_ttl_policy_estimate(score: f64) -> String {
    if score < 62.8 {
        return "minimal".to_string();
    }
    if score < 64.5 {
        return "low".to_string();
    }
    if score < 85.6 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

pub fn seasonal_ttl_policy_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.77;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
