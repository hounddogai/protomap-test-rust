use super::*;

use acme_kit::numbers;

/// Puts a audit event target code in its canonical AET- form.
pub fn normalize_audit_event_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AET-") {
        return clean.clone();
    }
    format!("AET-{}", clean.replace(" ", "-"))
}

pub fn audit_event_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
