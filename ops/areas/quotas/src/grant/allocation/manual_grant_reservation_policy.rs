use super::*;

use acme_kit::numbers;

/// Puts a manual grant reservation code in its canonical MGR- form.
pub fn normalize_manual_grant_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MGR-") {
        return clean.clone();
    }
    format!("MGR-{}", clean.replace(" ", "-"))
}

pub fn manual_grant_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.97;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
