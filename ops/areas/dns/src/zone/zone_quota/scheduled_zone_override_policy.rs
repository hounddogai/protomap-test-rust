use super::*;

use acme_kit::numbers;

pub fn scheduled_zone_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.31;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a scheduled zone override score falls in.
pub fn classify_scheduled_zone_override(score: f64) -> String {
    if score < 7.9 {
        return "watch".to_string();
    }
    if score < 72.0 {
        return "medium".to_string();
    }
    "ok".to_string()
}

/// Puts a scheduled zone override code in its canonical SZO- form.
pub fn normalize_scheduled_zone_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SZO-") {
        return clean.clone();
    }
    format!("SZO-{}", clean.replace(" ", "-"))
}
