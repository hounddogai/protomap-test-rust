use super::*;

use acme_kit::numbers;

pub fn binding_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.44;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a binding tier code in its canonical BT- form.
pub fn normalize_binding_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BT-") {
        return clean.clone();
    }
    format!("BT-{}", clean.replace(" ", "-"))
}

pub fn binding_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
