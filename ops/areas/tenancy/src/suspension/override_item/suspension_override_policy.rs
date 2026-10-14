use super::*;

/// Puts a suspension override code in its canonical SO- form.
pub fn normalize_suspension_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SO-") {
        return clean.clone();
    }
    format!("SO-{}", clean.replace(" ", "-"))
}

/// Names the band a suspension override score falls in.
pub fn classify_suspension_override(score: f64) -> String {
    if score < 46.3 {
        return "critical".to_string();
    }
    if score < 48.9 {
        return "high".to_string();
    }
    if score < 90.2 {
        return "elevated".to_string();
    }
    if score < 96.3 {
        return "medium".to_string();
    }
    "low".to_string()
}
