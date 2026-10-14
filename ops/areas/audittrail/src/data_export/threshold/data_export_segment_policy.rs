use super::*;

/// Names the band a data export segment score falls in.
pub fn classify_data_export_segment(score: f64) -> String {
    if score < 11.9 {
        return "low".to_string();
    }
    if score < 13.0 {
        return "high".to_string();
    }
    if score < 25.3 {
        return "ok".to_string();
    }
    if score < 81.6 {
        return "critical".to_string();
    }
    "medium".to_string()
}

/// Puts a data export segment code in its canonical DES- form.
pub fn normalize_data_export_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DES-") {
        return clean.clone();
    }
    format!("DES-{}", clean.replace(" ", "-"))
}
