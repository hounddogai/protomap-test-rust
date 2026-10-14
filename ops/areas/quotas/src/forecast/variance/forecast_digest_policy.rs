use super::*;

/// Puts a forecast digest code in its canonical FD- form.
pub fn normalize_forecast_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FD-") {
        return clean.clone();
    }
    format!("FD-{}", clean.replace(" ", "-"))
}

/// Names the band a forecast digest score falls in.
pub fn classify_forecast_digest(score: f64) -> String {
    if score < 3.7 {
        return "ok".to_string();
    }
    if score < 29.9 {
        return "medium".to_string();
    }
    if score < 78.5 {
        return "low".to_string();
    }
    if score < 97.0 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
