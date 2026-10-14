use super::*;

use acme_kit::numbers;

pub fn flow_log_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn flow_log_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.26;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a flow log tier score falls in.
pub fn classify_flow_log_tier(score: f64) -> String {
    if score < 45.1 {
        return "elevated".to_string();
    }
    if score < 47.0 {
        return "low".to_string();
    }
    if score < 71.7 {
        return "high".to_string();
    }
    if score < 91.8 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
