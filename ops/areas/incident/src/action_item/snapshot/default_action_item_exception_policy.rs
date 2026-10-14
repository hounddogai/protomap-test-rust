use super::*;

use acme_kit::numbers;

pub fn default_action_item_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a default action item exception code in its canonical DAIE- form.
pub fn normalize_default_action_item_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DAIE-") {
        return clean.clone();
    }
    format!("DAIE-{}", clean.replace(" ", "-"))
}
