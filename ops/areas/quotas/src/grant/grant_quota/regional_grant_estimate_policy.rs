use super::*;

use acme_kit::numbers;

pub fn regional_grant_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.54;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a regional grant estimate code in its canonical RGE- form.
pub fn normalize_regional_grant_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RGE-") {
        return clean.clone();
    }
    format!("RGE-{}", clean.replace(" ", "-"))
}
