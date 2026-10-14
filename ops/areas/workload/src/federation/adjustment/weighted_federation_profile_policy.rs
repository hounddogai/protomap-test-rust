use super::*;

use acme_kit::numbers;

pub fn weighted_federation_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a weighted federation profile score falls in.
pub fn classify_weighted_federation_profile(score: f64) -> String {
    if score < 38.9 {
        return "low".to_string();
    }
    if score < 53.2 {
        return "watch".to_string();
    }
    if score < 55.3 {
        return "ok".to_string();
    }
    if score < 80.0 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
