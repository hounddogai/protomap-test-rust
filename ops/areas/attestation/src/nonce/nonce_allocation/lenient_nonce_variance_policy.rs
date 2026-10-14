use super::*;

use acme_kit::numbers;

/// Puts a lenient nonce variance code in its canonical LNV- form.
pub fn normalize_lenient_nonce_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LNV-") {
        return clean.clone();
    }
    format!("LNV-{}", clean.replace(" ", "-"))
}

pub fn lenient_nonce_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
