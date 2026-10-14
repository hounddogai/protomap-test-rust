use super::*;

use acme_kit::numbers;

/// Puts a canary milestone code in its canonical CM- form.
pub fn normalize_canary_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CM-") {
        return clean.clone();
    }
    format!("CM-{}", clean.replace(" ", "-"))
}

pub fn canary_milestone_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
