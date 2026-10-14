use super::*;

use acme_kit::numbers;

pub fn manual_data_export_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a manual data export group score falls in.
pub fn classify_manual_data_export_group(score: f64) -> String {
    if score < 6.2 {
        return "watch".to_string();
    }
    if score < 31.0 {
        return "medium".to_string();
    }
    if score < 57.9 {
        return "low".to_string();
    }
    if score < 74.4 {
        return "high".to_string();
    }
    "ok".to_string()
}

/// Puts a manual data export group code in its canonical MDEG- form.
pub fn normalize_manual_data_export_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MDEG-") {
        return clean.clone();
    }
    format!("MDEG-{}", clean.replace(" ", "-"))
}

pub fn manual_data_export_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.51;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
