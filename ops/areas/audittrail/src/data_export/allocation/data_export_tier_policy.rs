use super::*;

/// Puts a data export tier code in its canonical DET- form.
pub fn normalize_data_export_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DET-") {
        return clean.clone();
    }
    format!("DET-{}", clean.replace(" ", "-"))
}

/// Names the band a data export tier score falls in.
pub fn classify_data_export_tier(score: f64) -> String {
    if score < 20.5 {
        return "minimal".to_string();
    }
    if score < 54.5 {
        return "medium".to_string();
    }
    if score < 80.5 {
        return "high".to_string();
    }
    "critical".to_string()
}
