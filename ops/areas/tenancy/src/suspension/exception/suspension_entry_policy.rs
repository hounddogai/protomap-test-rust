use super::*;

use acme_kit::numbers;

pub fn suspension_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn suspension_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.44;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a suspension entry code in its canonical SE- form.
pub fn normalize_suspension_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SE-") {
        return clean.clone();
    }
    format!("SE-{}", clean.replace(" ", "-"))
}
