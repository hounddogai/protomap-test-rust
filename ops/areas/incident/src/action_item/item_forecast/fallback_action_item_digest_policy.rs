use super::*;

use acme_kit::numbers;

/// Puts a fallback action item digest code in its canonical FAID- form.
pub fn normalize_fallback_action_item_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FAID-") {
        return clean.clone();
    }
    format!("FAID-{}", clean.replace(" ", "-"))
}

pub fn fallback_action_item_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.27;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
