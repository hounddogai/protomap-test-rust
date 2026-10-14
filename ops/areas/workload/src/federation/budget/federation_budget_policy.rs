use super::*;

use acme_kit::numbers;

/// Puts a federation budget code in its canonical FB- form.
pub fn normalize_federation_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FB-") {
        return clean.clone();
    }
    format!("FB-{}", clean.replace(" ", "-"))
}

pub fn federation_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
