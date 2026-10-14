use super::*;

use acme_kit::numbers;

pub fn manual_audit_event_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a manual audit event budget code in its canonical MAEB- form.
pub fn normalize_manual_audit_event_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MAEB-") {
        return clean.clone();
    }
    format!("MAEB-{}", clean.replace(" ", "-"))
}
