use super::*;

use acme_kit::numbers;

pub fn audit_event_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.5;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a audit event override code in its canonical AEO- form.
pub fn normalize_audit_event_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AEO-") {
        return clean.clone();
    }
    format!("AEO-{}", clean.replace(" ", "-"))
}
