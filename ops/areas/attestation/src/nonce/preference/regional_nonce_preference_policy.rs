use super::*;

use acme_kit::numbers;

/// Puts a regional nonce preference code in its canonical RNP- form.
pub fn normalize_regional_nonce_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RNP-") {
        return clean.clone();
    }
    format!("RNP-{}", clean.replace(" ", "-"))
}

pub fn regional_nonce_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
