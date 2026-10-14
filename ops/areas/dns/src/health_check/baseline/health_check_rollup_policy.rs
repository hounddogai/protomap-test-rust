use super::*;

use acme_kit::numbers;

/// Names the band a health check rollup score falls in.
pub fn classify_health_check_rollup(score: f64) -> String {
    if score < 3.2 {
        return "minimal".to_string();
    }
    if score < 87.2 {
        return "critical".to_string();
    }
    if score < 90.3 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn health_check_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.86;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
