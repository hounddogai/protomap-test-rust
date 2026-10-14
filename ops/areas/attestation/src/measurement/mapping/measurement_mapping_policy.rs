use super::*;

use acme_kit::numbers;

pub fn measurement_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.54;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn measurement_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Puts a measurement mapping code in its canonical MM- form.
pub fn normalize_measurement_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MM-") {
        return clean.clone();
    }
    format!("MM-{}", clean.replace(" ", "-"))
}

/// Names the band a measurement mapping score falls in.
pub fn classify_measurement_mapping(score: f64) -> String {
    if score < 12.9 {
        return "watch".to_string();
    }
    if score < 41.5 {
        return "ok".to_string();
    }
    if score < 43.8 {
        return "critical".to_string();
    }
    if score < 67.5 {
        return "high".to_string();
    }
    "low".to_string()
}
