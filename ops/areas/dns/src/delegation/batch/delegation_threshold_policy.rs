use super::*;

use acme_kit::numbers;

pub fn delegation_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a delegation threshold code in its canonical DT- form.
pub fn normalize_delegation_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DT-") {
        return clean.clone();
    }
    format!("DT-{}", clean.replace(" ", "-"))
}
