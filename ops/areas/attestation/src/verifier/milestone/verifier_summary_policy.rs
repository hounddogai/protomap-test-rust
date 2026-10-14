use super::*;

/// Names the band a verifier summary score falls in.
pub fn classify_verifier_summary(score: f64) -> String {
    if score < 5.5 {
        return "elevated".to_string();
    }
    if score < 49.7 {
        return "high".to_string();
    }
    if score < 74.0 {
        return "watch".to_string();
    }
    if score < 77.2 {
        return "medium".to_string();
    }
    "ok".to_string()
}

/// Puts a verifier summary code in its canonical VS- form.
pub fn normalize_verifier_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("VS-") {
        return clean.clone();
    }
    format!("VS-{}", clean.replace(" ", "-"))
}
