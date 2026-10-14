use super::*;

use acme_kit::numbers;

/// Puts a primary escalation slot code in its canonical PES- form.
pub fn normalize_primary_escalation_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PES-") {
        return clean.clone();
    }
    format!("PES-{}", clean.replace(" ", "-"))
}

pub fn primary_escalation_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.27;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
