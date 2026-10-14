use super::*;

use acme_kit::numbers;

pub fn regional_redaction_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a regional redaction template code in its canonical RRT- form.
pub fn normalize_regional_redaction_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRT-") {
        return clean.clone();
    }
    format!("RRT-{}", clean.replace(" ", "-"))
}
