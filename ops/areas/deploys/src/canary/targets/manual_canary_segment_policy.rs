use super::*;

use acme_kit::numbers;

/// Puts a manual canary segment code in its canonical MCS- form.
pub fn normalize_manual_canary_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MCS-") {
        return clean.clone();
    }
    format!("MCS-{}", clean.replace(" ", "-"))
}

pub fn manual_canary_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

pub fn manual_canary_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.44;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
