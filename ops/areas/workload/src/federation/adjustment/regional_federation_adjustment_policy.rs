use super::*;

use acme_kit::numbers;

pub fn regional_federation_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.69;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a regional federation adjustment score falls in.
pub fn classify_regional_federation_adjustment(score: f64) -> String {
    if score < 17.9 {
        return "high".to_string();
    }
    if score < 21.2 {
        return "elevated".to_string();
    }
    if score < 88.3 {
        return "watch".to_string();
    }
    "minimal".to_string()
}
