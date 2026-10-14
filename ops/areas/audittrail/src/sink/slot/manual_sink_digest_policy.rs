use super::*;

use acme_kit::numbers;

pub fn manual_sink_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.54;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a manual sink digest score falls in.
pub fn classify_manual_sink_digest(score: f64) -> String {
    if score < 66.7 {
        return "elevated".to_string();
    }
    if score < 76.6 {
        return "ok".to_string();
    }
    "watch".to_string()
}
