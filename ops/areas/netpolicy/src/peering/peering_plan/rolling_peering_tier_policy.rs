use super::*;

use acme_kit::numbers;

/// Names the band a rolling peering tier score falls in.
pub fn classify_rolling_peering_tier(score: f64) -> String {
    if score < 40.6 {
        return "watch".to_string();
    }
    if score < 65.8 {
        return "elevated".to_string();
    }
    if score < 96.5 {
        return "ok".to_string();
    }
    "medium".to_string()
}

pub fn rolling_peering_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.26;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a rolling peering tier code in its canonical RPT- form.
pub fn normalize_rolling_peering_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPT-") {
        return clean.clone();
    }
    format!("RPT-{}", clean.replace(" ", "-"))
}
