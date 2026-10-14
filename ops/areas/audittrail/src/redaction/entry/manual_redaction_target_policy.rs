use super::*;

use acme_kit::numbers;

pub fn manual_redaction_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a manual redaction target code in its canonical MRT- form.
pub fn normalize_manual_redaction_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MRT-") {
        return clean.clone();
    }
    format!("MRT-{}", clean.replace(" ", "-"))
}
