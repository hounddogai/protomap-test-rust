use super::*;

use acme_kit::numbers;

pub fn delegation_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.05;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a delegation window code in its canonical DW- form.
pub fn normalize_delegation_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DW-") {
        return clean.clone();
    }
    format!("DW-{}", clean.replace(" ", "-"))
}

pub fn delegation_window_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
