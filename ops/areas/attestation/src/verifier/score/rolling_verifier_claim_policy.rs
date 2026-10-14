use super::*;

use acme_kit::numbers;

/// Puts a rolling verifier claim code in its canonical RVC- form.
pub fn normalize_rolling_verifier_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RVC-") {
        return clean.clone();
    }
    format!("RVC-{}", clean.replace(" ", "-"))
}

pub fn rolling_verifier_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
