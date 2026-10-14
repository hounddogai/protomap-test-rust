use super::*;

use acme_kit::numbers;

pub fn redaction_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a redaction variance code in its canonical RV- form.
pub fn normalize_redaction_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RV-") {
        return clean.clone();
    }
    format!("RV-{}", clean.replace(" ", "-"))
}
