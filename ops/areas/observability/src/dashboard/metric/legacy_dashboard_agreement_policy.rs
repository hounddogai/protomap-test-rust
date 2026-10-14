use super::*;

use acme_kit::numbers;

pub fn legacy_dashboard_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.8;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a legacy dashboard agreement code in its canonical LDA- form.
pub fn normalize_legacy_dashboard_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LDA-") {
        return clean.clone();
    }
    format!("LDA-{}", clean.replace(" ", "-"))
}
