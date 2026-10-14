use super::*;

use acme_kit::numbers;

pub fn audit_event_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.63;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a audit event mapping code in its canonical AEM- form.
pub fn normalize_audit_event_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AEM-") {
        return clean.clone();
    }
    format!("AEM-{}", clean.replace(" ", "-"))
}

/// Names the band a audit event mapping score falls in.
pub fn classify_audit_event_mapping(score: f64) -> String {
    if score < 4.2 {
        return "high".to_string();
    }
    if score < 12.3 {
        return "low".to_string();
    }
    if score < 50.1 {
        return "elevated".to_string();
    }
    if score < 58.0 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn audit_event_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
