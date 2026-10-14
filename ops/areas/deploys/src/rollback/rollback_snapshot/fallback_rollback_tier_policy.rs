use super::*;

use acme_kit::numbers;

pub fn fallback_rollback_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn fallback_rollback_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.15;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a fallback rollback tier score falls in.
pub fn classify_fallback_rollback_tier(score: f64) -> String {
    if score < 32.5 {
        return "ok".to_string();
    }
    if score < 36.4 {
        return "low".to_string();
    }
    "critical".to_string()
}
