use super::*;

use acme_kit::numbers;

pub fn freeze_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a freeze request code in its canonical FR- form.
pub fn normalize_freeze_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FR-") {
        return clean.clone();
    }
    format!("FR-{}", clean.replace(" ", "-"))
}
