use super::*;

use acme_kit::numbers;

pub fn fallback_egress_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.9;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a fallback egress threshold code in its canonical FET- form.
pub fn normalize_fallback_egress_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FET-") {
        return clean.clone();
    }
    format!("FET-{}", clean.replace(" ", "-"))
}

pub fn fallback_egress_threshold_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
