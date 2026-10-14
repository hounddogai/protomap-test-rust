use super::*;

use acme_kit::numbers;

pub fn usage_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a usage claim score falls in.
pub fn classify_usage_claim(score: f64) -> String {
    if score < 10.8 {
        return "elevated".to_string();
    }
    if score < 16.0 {
        return "ok".to_string();
    }
    if score < 35.0 {
        return "critical".to_string();
    }
    if score < 56.1 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn usage_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.89;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn usage_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
