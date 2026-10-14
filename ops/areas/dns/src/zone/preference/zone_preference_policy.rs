use super::*;

use acme_kit::numbers;

/// Puts a zone preference code in its canonical ZP- form.
pub fn normalize_zone_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ZP-") {
        return clean.clone();
    }
    format!("ZP-{}", clean.replace(" ", "-"))
}

pub fn zone_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
