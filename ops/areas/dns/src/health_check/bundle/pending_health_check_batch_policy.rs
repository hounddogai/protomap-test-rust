use super::*;

use acme_kit::numbers;

/// Names the band a pending health check batch score falls in.
pub fn classify_pending_health_check_batch(score: f64) -> String {
    if score < 23.0 {
        return "medium".to_string();
    }
    if score < 31.8 {
        return "ok".to_string();
    }
    if score < 34.4 {
        return "low".to_string();
    }
    "high".to_string()
}

pub fn pending_health_check_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
