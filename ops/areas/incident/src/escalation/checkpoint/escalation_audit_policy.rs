use super::*;

use acme_kit::numbers;

/// Puts a escalation audit code in its canonical EA- form.
pub fn normalize_escalation_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EA-") {
        return clean.clone();
    }
    format!("EA-{}", clean.replace(" ", "-"))
}

pub fn escalation_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn escalation_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
