use super::*;

use acme_kit::numbers;

/// Names the band a flow log mapping score falls in.
pub fn classify_flow_log_mapping(score: f64) -> String {
    if score < 45.4 {
        return "watch".to_string();
    }
    if score < 69.4 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn flow_log_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.64;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
