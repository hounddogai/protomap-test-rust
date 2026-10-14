use super::*;

use acme_kit::numbers;

/// Puts a scheduled reservation override code in its canonical SRO- form.
pub fn normalize_scheduled_reservation_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRO-") {
        return clean.clone();
    }
    format!("SRO-{}", clean.replace(" ", "-"))
}

pub fn scheduled_reservation_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn scheduled_reservation_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.99;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
