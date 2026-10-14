use super::*;

use acme_kit::numbers;

pub fn measurement_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a measurement budget code in its canonical MB- form.
pub fn normalize_measurement_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MB-") {
        return clean.clone();
    }
    format!("MB-{}", clean.replace(" ", "-"))
}
