use super::*;

use acme_kit::numbers;

pub fn slo_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.47;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a slo record score falls in.
pub fn classify_slo_record(score: f64) -> String {
    if score < 54.9 {
        return "minimal".to_string();
    }
    if score < 88.7 {
        return "high".to_string();
    }
    "ok".to_string()
}
