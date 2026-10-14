use super::*;

use acme_kit::numbers;

/// Puts a verifier forecast code in its canonical VF- form.
pub fn normalize_verifier_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("VF-") {
        return clean.clone();
    }
    format!("VF-{}", clean.replace(" ", "-"))
}

pub fn verifier_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
