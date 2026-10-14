use super::*;

/// Puts a usage manifest code in its canonical UM- form.
pub fn normalize_usage_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("UM-") {
        return clean.clone();
    }
    format!("UM-{}", clean.replace(" ", "-"))
}

/// Names the band a usage manifest score falls in.
pub fn classify_usage_manifest(score: f64) -> String {
    if score < 4.6 {
        return "medium".to_string();
    }
    if score < 24.7 {
        return "critical".to_string();
    }
    if score < 66.6 {
        return "high".to_string();
    }
    "low".to_string()
}
