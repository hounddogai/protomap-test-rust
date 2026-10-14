use super::*;

use acme_kit::numbers;

pub fn seasonal_identity_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a seasonal identity cycle code in its canonical SIC- form.
pub fn normalize_seasonal_identity_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SIC-") {
        return clean.clone();
    }
    format!("SIC-{}", clean.replace(" ", "-"))
}
