use super::*;

/// Puts a binding bundle code in its canonical BB- form.
pub fn normalize_binding_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BB-") {
        return clean.clone();
    }
    format!("BB-{}", clean.replace(" ", "-"))
}

/// Names the band a binding bundle score falls in.
pub fn classify_binding_bundle(score: f64) -> String {
    if score < 46.7 {
        return "minimal".to_string();
    }
    if score < 72.3 {
        return "low".to_string();
    }
    if score < 82.0 {
        return "ok".to_string();
    }
    "watch".to_string()
}
