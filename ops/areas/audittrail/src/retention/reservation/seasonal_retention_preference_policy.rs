use super::*;

/// Names the band a seasonal retention preference score falls in.
pub fn classify_seasonal_retention_preference(score: f64) -> String {
    if score < 48.7 {
        return "low".to_string();
    }
    if score < 62.6 {
        return "medium".to_string();
    }
    "watch".to_string()
}

/// Puts a seasonal retention preference code in its canonical SRP- form.
pub fn normalize_seasonal_retention_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRP-") {
        return clean.clone();
    }
    format!("SRP-{}", clean.replace(" ", "-"))
}
