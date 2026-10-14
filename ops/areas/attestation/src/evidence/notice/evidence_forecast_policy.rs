use super::*;

use acme_kit::numbers;

/// Puts a evidence forecast code in its canonical EF- form.
pub fn normalize_evidence_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EF-") {
        return clean.clone();
    }
    format!("EF-{}", clean.replace(" ", "-"))
}

pub fn evidence_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.09;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
