use super::*;

use acme_kit::numbers;

/// Puts a data export threshold code in its canonical DET- form.
pub fn normalize_data_export_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DET-") {
        return clean.clone();
    }
    format!("DET-{}", clean.replace(" ", "-"))
}

pub fn data_export_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.61;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a data export threshold score falls in.
pub fn classify_data_export_threshold(score: f64) -> String {
    if score < 15.1 {
        return "ok".to_string();
    }
    if score < 52.4 {
        return "low".to_string();
    }
    if score < 92.6 {
        return "critical".to_string();
    }
    "watch".to_string()
}
