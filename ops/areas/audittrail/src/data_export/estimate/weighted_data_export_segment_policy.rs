use super::*;

use acme_kit::numbers;

/// Puts a weighted data export segment code in its canonical WDES- form.
pub fn normalize_weighted_data_export_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WDES-") {
        return clean.clone();
    }
    format!("WDES-{}", clean.replace(" ", "-"))
}

pub fn weighted_data_export_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn weighted_data_export_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn weighted_data_export_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.68;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
