use super::*;

use acme_kit::numbers;

pub fn ttl_policy_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a ttl policy manifest code in its canonical TPM- form.
pub fn normalize_ttl_policy_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TPM-") {
        return clean.clone();
    }
    format!("TPM-{}", clean.replace(" ", "-"))
}
