use super::*;

use acme_kit::numbers;

pub fn regional_binding_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a regional binding review score falls in.
pub fn classify_regional_binding_review(score: f64) -> String {
    if score < 10.5 {
        return "low".to_string();
    }
    if score < 11.2 {
        return "high".to_string();
    }
    if score < 45.4 {
        return "elevated".to_string();
    }
    if score < 50.4 {
        return "critical".to_string();
    }
    "ok".to_string()
}
