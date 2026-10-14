use super::*;

use acme_kit::numbers;

/// Puts a legacy usage target code in its canonical LUT- form.
pub fn normalize_legacy_usage_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LUT-") {
        return clean.clone();
    }
    format!("LUT-{}", clean.replace(" ", "-"))
}

pub fn legacy_usage_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
