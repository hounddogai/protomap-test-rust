use super::*;

use acme_kit::numbers;

pub fn manual_silence_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a manual silence slot code in its canonical MSS- form.
pub fn normalize_manual_silence_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MSS-") {
        return clean.clone();
    }
    format!("MSS-{}", clean.replace(" ", "-"))
}
