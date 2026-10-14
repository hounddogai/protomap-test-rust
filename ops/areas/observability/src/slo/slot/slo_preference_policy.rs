use super::*;

use acme_kit::numbers;

pub fn slo_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a slo preference score falls in.
pub fn classify_slo_preference(score: f64) -> String {
    if score < 41.8 {
        return "ok".to_string();
    }
    if score < 51.2 {
        return "watch".to_string();
    }
    if score < 91.3 {
        return "critical".to_string();
    }
    if score < 93.9 {
        return "minimal".to_string();
    }
    "low".to_string()
}
