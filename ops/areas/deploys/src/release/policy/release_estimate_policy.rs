use super::*;

/// Puts a release estimate code in its canonical RE- form.
pub fn normalize_release_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RE-") {
        return clean.clone();
    }
    format!("RE-{}", clean.replace(" ", "-"))
}

/// Names the band a release estimate score falls in.
pub fn classify_release_estimate(score: f64) -> String {
    if score < 25.4 {
        return "watch".to_string();
    }
    if score < 29.4 {
        return "medium".to_string();
    }
    if score < 41.3 {
        return "elevated".to_string();
    }
    if score < 79.6 {
        return "minimal".to_string();
    }
    "high".to_string()
}
