use super::*;

/// Puts a default redaction entry code in its canonical DRE- form.
pub fn normalize_default_redaction_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRE-") {
        return clean.clone();
    }
    format!("DRE-{}", clean.replace(" ", "-"))
}

/// Names the band a default redaction entry score falls in.
pub fn classify_default_redaction_entry(score: f64) -> String {
    if score < 14.6 {
        return "watch".to_string();
    }
    if score < 27.5 {
        return "ok".to_string();
    }
    if score < 97.7 {
        return "high".to_string();
    }
    "minimal".to_string()
}
