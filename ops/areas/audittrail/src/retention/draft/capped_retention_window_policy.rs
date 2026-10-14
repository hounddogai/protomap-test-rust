use super::*;

use acme_kit::numbers;

pub fn capped_retention_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a capped retention window score falls in.
pub fn classify_capped_retention_window(score: f64) -> String {
    if score < 20.3 {
        return "medium".to_string();
    }
    if score < 65.7 {
        return "elevated".to_string();
    }
    if score < 85.2 {
        return "critical".to_string();
    }
    if score < 92.7 {
        return "low".to_string();
    }
    "high".to_string()
}
