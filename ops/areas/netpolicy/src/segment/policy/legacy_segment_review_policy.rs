use super::*;

use acme_kit::numbers;

pub fn legacy_segment_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn legacy_segment_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a legacy segment review code in its canonical LSR- form.
pub fn normalize_legacy_segment_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LSR-") {
        return clean.clone();
    }
    format!("LSR-{}", clean.replace(" ", "-"))
}
