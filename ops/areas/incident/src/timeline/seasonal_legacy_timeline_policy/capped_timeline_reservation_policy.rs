use super::*;

use acme_kit::numbers;

pub fn capped_timeline_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.3;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a capped timeline reservation score falls in.
pub fn classify_capped_timeline_reservation(score: f64) -> String {
    if score < 40.0 {
        return "high".to_string();
    }
    if score < 72.2 {
        return "elevated".to_string();
    }
    if score < 80.8 {
        return "medium".to_string();
    }
    if score < 98.8 {
        return "ok".to_string();
    }
    "low".to_string()
}
