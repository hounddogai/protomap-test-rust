use super::*;

/// Puts a fallback nonce estimate code in its canonical FNE- form.
pub fn normalize_fallback_nonce_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FNE-") {
        return clean.clone();
    }
    format!("FNE-{}", clean.replace(" ", "-"))
}

/// Names the band a fallback nonce estimate score falls in.
pub fn classify_fallback_nonce_estimate(score: f64) -> String {
    if score < 23.1 {
        return "watch".to_string();
    }
    if score < 28.1 {
        return "critical".to_string();
    }
    if score < 34.3 {
        return "high".to_string();
    }
    if score < 69.6 {
        return "minimal".to_string();
    }
    "medium".to_string()
}
