use super::*;

/// Names the band a peering request score falls in.
pub fn classify_peering_request(score: f64) -> String {
    if score < 11.8 {
        return "critical".to_string();
    }
    if score < 56.0 {
        return "watch".to_string();
    }
    if score < 76.9 {
        return "ok".to_string();
    }
    if score < 82.4 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Puts a peering request code in its canonical PR- form.
pub fn normalize_peering_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PR-") {
        return clean.clone();
    }
    format!("PR-{}", clean.replace(" ", "-"))
}
