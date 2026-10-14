use super::*;

use acme_kit::numbers;

pub fn weighted_runbook_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.78;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a weighted runbook request code in its canonical WRR- form.
pub fn normalize_weighted_runbook_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WRR-") {
        return clean.clone();
    }
    format!("WRR-{}", clean.replace(" ", "-"))
}
