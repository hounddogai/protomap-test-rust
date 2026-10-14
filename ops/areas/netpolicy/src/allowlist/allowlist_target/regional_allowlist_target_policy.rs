use super::*;

use acme_kit::numbers;

pub fn regional_allowlist_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a regional allowlist target code in its canonical RAT- form.
pub fn normalize_regional_allowlist_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RAT-") {
        return clean.clone();
    }
    format!("RAT-{}", clean.replace(" ", "-"))
}
