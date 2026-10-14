use super::*;

use acme_kit::numbers;

/// Names the band a strict region pin tier score falls in.
pub fn classify_strict_region_pin_tier(score: f64) -> String {
    if score < 31.4 {
        return "minimal".to_string();
    }
    if score < 61.5 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn strict_region_pin_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
