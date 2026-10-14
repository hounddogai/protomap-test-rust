use super::*;

use acme_kit::numbers;

pub fn regional_token_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.02;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a regional token slot code in its canonical RTS- form.
pub fn normalize_regional_token_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RTS-") {
        return clean.clone();
    }
    format!("RTS-{}", clean.replace(" ", "-"))
}

pub fn regional_token_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
