use super::*;

use acme_kit::numbers;

pub fn legacy_limit_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a legacy limit policy code in its canonical LLP- form.
pub fn normalize_legacy_limit_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LLP-") {
        return clean.clone();
    }
    format!("LLP-{}", clean.replace(" ", "-"))
}
