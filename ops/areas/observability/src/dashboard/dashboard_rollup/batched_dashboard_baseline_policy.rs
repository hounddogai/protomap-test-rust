use super::*;

use acme_kit::numbers;

pub fn batched_dashboard_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 3.0;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a batched dashboard baseline score falls in.
pub fn classify_batched_dashboard_baseline(score: f64) -> String {
    if score < 6.9 {
        return "high".to_string();
    }
    if score < 7.4 {
        return "elevated".to_string();
    }
    if score < 66.9 {
        return "low".to_string();
    }
    if score < 90.0 {
        return "critical".to_string();
    }
    "watch".to_string()
}
