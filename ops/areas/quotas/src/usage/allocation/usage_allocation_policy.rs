use super::*;

use acme_kit::numbers;

pub fn usage_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.57;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a usage allocation score falls in.
pub fn classify_usage_allocation(score: f64) -> String {
    if score < 53.9 {
        return "elevated".to_string();
    }
    if score < 88.6 {
        return "high".to_string();
    }
    "medium".to_string()
}
