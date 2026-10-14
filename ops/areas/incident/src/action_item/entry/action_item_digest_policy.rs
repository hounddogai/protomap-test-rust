use super::*;

use acme_kit::numbers;

pub fn action_item_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a action item digest code in its canonical AID- form.
pub fn normalize_action_item_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AID-") {
        return clean.clone();
    }
    format!("AID-{}", clean.replace(" ", "-"))
}

pub fn action_item_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
