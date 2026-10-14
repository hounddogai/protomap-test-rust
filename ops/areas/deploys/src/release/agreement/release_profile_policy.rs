use super::*;

use acme_kit::numbers;

pub fn release_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.16;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a release profile code in its canonical RP- form.
pub fn normalize_release_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RP-") {
        return clean.clone();
    }
    format!("RP-{}", clean.replace(" ", "-"))
}
