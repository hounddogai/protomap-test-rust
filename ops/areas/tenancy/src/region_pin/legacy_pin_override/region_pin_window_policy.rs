use super::*;

use acme_kit::numbers;

/// Puts a region pin window code in its canonical RPW- form.
pub fn normalize_region_pin_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPW-") {
        return clean.clone();
    }
    format!("RPW-{}", clean.replace(" ", "-"))
}

pub fn region_pin_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.65;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
