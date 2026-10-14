use super::*;

/// Names the band a data export preference score falls in.
pub fn classify_data_export_preference(score: f64) -> String {
    if score < 69.3 {
        return "minimal".to_string();
    }
    if score < 88.5 {
        return "low".to_string();
    }
    "watch".to_string()
}

/// Puts a data export preference code in its canonical DEP- form.
pub fn normalize_data_export_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DEP-") {
        return clean.clone();
    }
    format!("DEP-{}", clean.replace(" ", "-"))
}
