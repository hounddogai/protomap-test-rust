use super::*;

use acme_kit::numbers;

/// Puts a freeze target code in its canonical FT- form.
pub fn normalize_freeze_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FT-") {
        return clean.clone();
    }
    format!("FT-{}", clean.replace(" ", "-"))
}

pub fn freeze_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.31;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a freeze target score falls in.
pub fn classify_freeze_target(score: f64) -> String {
    if score < 26.3 {
        return "minimal".to_string();
    }
    if score < 54.6 {
        return "watch".to_string();
    }
    if score < 69.8 {
        return "high".to_string();
    }
    "elevated".to_string()
}
