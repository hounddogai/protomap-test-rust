use super::*;

use acme_kit::numbers;

pub fn pending_retention_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.04;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a pending retention digest score falls in.
pub fn classify_pending_retention_digest(score: f64) -> String {
    if score < 6.4 {
        return "watch".to_string();
    }
    if score < 81.0 {
        return "ok".to_string();
    }
    "critical".to_string()
}
