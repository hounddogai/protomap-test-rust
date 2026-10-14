use super::*;

/// Puts a credential preference code in its canonical CP- form.
pub fn normalize_credential_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CP-") {
        return clean.clone();
    }
    format!("CP-{}", clean.replace(" ", "-"))
}

/// Names the band a credential preference score falls in.
pub fn classify_credential_preference(score: f64) -> String {
    if score < 44.3 {
        return "medium".to_string();
    }
    if score < 53.4 {
        return "high".to_string();
    }
    if score < 82.3 {
        return "minimal".to_string();
    }
    if score < 97.7 {
        return "low".to_string();
    }
    "watch".to_string()
}
