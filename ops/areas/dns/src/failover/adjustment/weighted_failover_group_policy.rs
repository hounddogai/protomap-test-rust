use super::*;

use acme_kit::numbers;

/// Names the band a weighted failover group score falls in.
pub fn classify_weighted_failover_group(score: f64) -> String {
    if score < 76.5 {
        return "ok".to_string();
    }
    if score < 82.0 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn weighted_failover_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.43;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
