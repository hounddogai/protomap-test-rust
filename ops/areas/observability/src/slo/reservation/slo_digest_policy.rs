use super::*;

use acme_kit::numbers;

/// Names the band a slo digest score falls in.
pub fn classify_slo_digest(score: f64) -> String {
    if score < 8.2 {
        return "medium".to_string();
    }
    if score < 10.7 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn slo_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
