use super::*;

use acme_kit::numbers;

/// Puts a fallback binding claim code in its canonical FBC- form.
pub fn normalize_fallback_binding_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FBC-") {
        return clean.clone();
    }
    format!("FBC-{}", clean.replace(" ", "-"))
}

pub fn fallback_binding_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn fallback_binding_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.96;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
