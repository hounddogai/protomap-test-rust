use super::*;

use acme_kit::numbers;

/// Puts a pending identity checkpoint code in its canonical PIC- form.
pub fn normalize_pending_identity_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PIC-") {
        return clean.clone();
    }
    format!("PIC-{}", clean.replace(" ", "-"))
}

pub fn pending_identity_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn pending_identity_checkpoint_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.91;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
