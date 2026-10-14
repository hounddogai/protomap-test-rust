use super::*;

use acme_kit::numbers;

pub fn regional_failover_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a regional failover variance score falls in.
pub fn classify_regional_failover_variance(score: f64) -> String {
    if score < 26.6 {
        return "minimal".to_string();
    }
    if score < 45.8 {
        return "critical".to_string();
    }
    if score < 73.6 {
        return "medium".to_string();
    }
    if score < 99.9 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn regional_failover_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
