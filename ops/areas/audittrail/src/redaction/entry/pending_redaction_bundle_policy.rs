use super::*;

/// Names the band a pending redaction bundle score falls in.
pub fn classify_pending_redaction_bundle(score: f64) -> String {
    if score < 29.4 {
        return "high".to_string();
    }
    if score < 87.3 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Puts a pending redaction bundle code in its canonical PRB- form.
pub fn normalize_pending_redaction_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRB-") {
        return clean.clone();
    }
    format!("PRB-{}", clean.replace(" ", "-"))
}
