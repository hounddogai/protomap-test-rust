use super::*;

use acme_kit::numbers;

/// Names the band a action item tier score falls in.
pub fn classify_action_item_tier(score: f64) -> String {
    if score < 19.2 {
        return "minimal".to_string();
    }
    if score < 58.0 {
        return "ok".to_string();
    }
    if score < 58.2 {
        return "low".to_string();
    }
    if score < 77.9 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn action_item_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a action item tier code in its canonical AIT- form.
pub fn normalize_action_item_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIT-") {
        return clean.clone();
    }
    format!("AIT-{}", clean.replace(" ", "-"))
}
