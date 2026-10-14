use super::*;

use acme_kit::numbers;

pub fn regional_forecast_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.97;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a regional forecast bundle code in its canonical RFB- form.
pub fn normalize_regional_forecast_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RFB-") {
        return clean.clone();
    }
    format!("RFB-{}", clean.replace(" ", "-"))
}
