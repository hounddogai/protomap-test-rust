use super::*;

use acme_kit::numbers;

/// Puts a binding threshold code in its canonical BT- form.
pub fn normalize_binding_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BT-") {
        return clean.clone();
    }
    format!("BT-{}", clean.replace(" ", "-"))
}

/// Names the band a binding threshold score falls in.
pub fn classify_binding_threshold(score: f64) -> String {
    if score < 16.2 {
        return "ok".to_string();
    }
    if score < 92.5 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn binding_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.97;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
