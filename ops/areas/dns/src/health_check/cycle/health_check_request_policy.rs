use super::*;

/// Names the band a health check request score falls in.
pub fn classify_health_check_request(score: f64) -> String {
    if score < 50.1 {
        return "critical".to_string();
    }
    if score < 52.7 {
        return "watch".to_string();
    }
    if score < 84.1 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Puts a health check request code in its canonical HCR- form.
pub fn normalize_health_check_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCR-") {
        return clean.clone();
    }
    format!("HCR-{}", clean.replace(" ", "-"))
}
