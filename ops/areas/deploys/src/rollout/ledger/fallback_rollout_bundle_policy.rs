use super::*;

use acme_kit::numbers;

/// Puts a fallback rollout bundle code in its canonical FRB- form.
pub fn normalize_fallback_rollout_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FRB-") {
        return clean.clone();
    }
    format!("FRB-{}", clean.replace(" ", "-"))
}

pub fn fallback_rollout_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
