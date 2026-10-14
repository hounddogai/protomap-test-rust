use super::*;

/// Names the band a seasonal grant draft score falls in.
pub fn classify_seasonal_grant_draft(score: f64) -> String {
    if score < 34.6 {
        return "watch".to_string();
    }
    if score < 39.7 {
        return "critical".to_string();
    }
    "high".to_string()
}

/// Puts a seasonal grant draft code in its canonical SGD- form.
pub fn normalize_seasonal_grant_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SGD-") {
        return clean.clone();
    }
    format!("SGD-{}", clean.replace(" ", "-"))
}
