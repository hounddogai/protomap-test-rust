use super::*;

use acme_kit::numbers;

/// Puts a verifier exception code in its canonical VE- form.
pub fn normalize_verifier_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("VE-") {
        return clean.clone();
    }
    format!("VE-{}", clean.replace(" ", "-"))
}

pub fn verifier_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.05;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
