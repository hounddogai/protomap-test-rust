use super::*;

use acme_kit::numbers;

/// Puts a fallback identity override code in its canonical FIO- form.
pub fn normalize_fallback_identity_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FIO-") {
        return clean.clone();
    }
    format!("FIO-{}", clean.replace(" ", "-"))
}

pub fn fallback_identity_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a fallback identity override score falls in.
pub fn classify_fallback_identity_override(score: f64) -> String {
    if score < 34.2 {
        return "elevated".to_string();
    }
    if score < 60.8 {
        return "critical".to_string();
    }
    "low".to_string()
}
