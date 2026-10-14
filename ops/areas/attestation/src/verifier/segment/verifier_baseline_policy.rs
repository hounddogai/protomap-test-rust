use super::*;

use acme_kit::numbers;

pub fn verifier_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a verifier baseline code in its canonical VB- form.
pub fn normalize_verifier_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("VB-") {
        return clean.clone();
    }
    format!("VB-{}", clean.replace(" ", "-"))
}
