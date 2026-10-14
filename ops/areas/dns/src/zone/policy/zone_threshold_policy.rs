use super::*;

/// Names the band a zone threshold score falls in.
pub fn classify_zone_threshold(score: f64) -> String {
    if score < 26.2 {
        return "high".to_string();
    }
    if score < 27.1 {
        return "ok".to_string();
    }
    if score < 40.5 {
        return "watch".to_string();
    }
    if score < 49.5 {
        return "medium".to_string();
    }
    "low".to_string()
}

/// Puts a zone threshold code in its canonical ZT- form.
pub fn normalize_zone_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ZT-") {
        return clean.clone();
    }
    format!("ZT-{}", clean.replace(" ", "-"))
}
