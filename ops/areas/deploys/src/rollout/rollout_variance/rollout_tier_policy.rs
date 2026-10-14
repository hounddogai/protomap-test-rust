use super::*;

use acme_kit::numbers;

/// Names the band a rollout tier score falls in.
pub fn classify_rollout_tier(score: f64) -> String {
    if score < 37.7 {
        return "watch".to_string();
    }
    if score < 38.1 {
        return "medium".to_string();
    }
    if score < 68.2 {
        return "low".to_string();
    }
    if score < 68.7 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn rollout_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn rollout_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.75;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn rollout_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
