use super::*;

use acme_kit::numbers;

pub fn manual_forecast_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.82;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a manual forecast audit code in its canonical MFA- form.
pub fn normalize_manual_forecast_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MFA-") {
        return clean.clone();
    }
    format!("MFA-{}", clean.replace(" ", "-"))
}
