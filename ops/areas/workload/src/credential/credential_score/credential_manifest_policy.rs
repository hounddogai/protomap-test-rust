use super::*;

use acme_kit::numbers;

pub fn credential_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a credential manifest code in its canonical CM- form.
pub fn normalize_credential_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CM-") {
        return clean.clone();
    }
    format!("CM-{}", clean.replace(" ", "-"))
}
