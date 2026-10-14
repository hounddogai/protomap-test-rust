use super::*;

/// Names the band a alert estimate score falls in.
pub fn classify_alert_estimate(score: f64) -> String {
    if score < 37.7 {
        return "high".to_string();
    }
    if score < 40.1 {
        return "watch".to_string();
    }
    if score < 41.1 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Puts a alert estimate code in its canonical AE- form.
pub fn normalize_alert_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AE-") {
        return clean.clone();
    }
    format!("AE-{}", clean.replace(" ", "-"))
}
