use super::*;

use acme_kit::numbers;

/// Puts a pending nonce record code in its canonical PNR- form.
pub fn normalize_pending_nonce_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PNR-") {
        return clean.clone();
    }
    format!("PNR-{}", clean.replace(" ", "-"))
}

pub fn pending_nonce_record_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
