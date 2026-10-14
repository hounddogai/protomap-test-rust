use super::*;

use acme_kit::numbers;

/// Puts a default freeze profile code in its canonical DFP- form.
pub fn normalize_default_freeze_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DFP-") {
        return clean.clone();
    }
    format!("DFP-{}", clean.replace(" ", "-"))
}

pub fn default_freeze_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
