use super::*;

use acme_kit::numbers;

pub fn global_measurement_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.99;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a global measurement profile code in its canonical GMP- form.
pub fn normalize_global_measurement_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GMP-") {
        return clean.clone();
    }
    format!("GMP-{}", clean.replace(" ", "-"))
}
