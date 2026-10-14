use super::*;

use acme_kit::numbers;

pub fn legacy_freeze_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a legacy freeze audit code in its canonical LFA- form.
pub fn normalize_legacy_freeze_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LFA-") {
        return clean.clone();
    }
    format!("LFA-{}", clean.replace(" ", "-"))
}
