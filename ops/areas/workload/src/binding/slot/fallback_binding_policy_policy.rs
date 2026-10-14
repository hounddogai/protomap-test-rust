use super::*;

use acme_kit::numbers;

pub fn fallback_binding_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.53;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn fallback_binding_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a fallback binding policy code in its canonical FBP- form.
pub fn normalize_fallback_binding_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FBP-") {
        return clean.clone();
    }
    format!("FBP-{}", clean.replace(" ", "-"))
}
