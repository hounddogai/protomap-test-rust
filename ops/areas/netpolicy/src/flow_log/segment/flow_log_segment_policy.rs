use super::*;

use acme_kit::numbers;

pub fn flow_log_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.37;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a flow log segment score falls in.
pub fn classify_flow_log_segment(score: f64) -> String {
    if score < 22.9 {
        return "watch".to_string();
    }
    if score < 47.5 {
        return "medium".to_string();
    }
    if score < 49.1 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}
