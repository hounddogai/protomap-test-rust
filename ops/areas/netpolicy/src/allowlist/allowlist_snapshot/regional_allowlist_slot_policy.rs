use super::*;

use acme_kit::numbers;

/// Names the band a regional allowlist slot score falls in.
pub fn classify_regional_allowlist_slot(score: f64) -> String {
    if score < 60.7 {
        return "elevated".to_string();
    }
    if score < 62.5 {
        return "medium".to_string();
    }
    if score < 77.5 {
        return "critical".to_string();
    }
    if score < 89.1 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

pub fn regional_allowlist_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.37;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn regional_allowlist_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
