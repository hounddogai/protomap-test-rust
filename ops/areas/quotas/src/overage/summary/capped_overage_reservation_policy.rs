use super::*;

use acme_kit::numbers;

pub fn capped_overage_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.5;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a capped overage reservation score falls in.
pub fn classify_capped_overage_reservation(score: f64) -> String {
    if score < 22.3 {
        return "high".to_string();
    }
    if score < 43.5 {
        return "ok".to_string();
    }
    if score < 95.3 {
        return "low".to_string();
    }
    if score < 96.8 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
