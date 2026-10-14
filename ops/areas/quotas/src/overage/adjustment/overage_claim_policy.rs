use super::*;

use acme_kit::numbers;

/// Names the band a overage claim score falls in.
pub fn classify_overage_claim(score: f64) -> String {
    if score < 8.6 {
        return "medium".to_string();
    }
    if score < 37.1 {
        return "ok".to_string();
    }
    if score < 61.6 {
        return "high".to_string();
    }
    "minimal".to_string()
}

pub fn overage_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn overage_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.27;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn overage_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
