use super::*;

/// Names the band a pending audit event profile score falls in.
pub fn classify_pending_audit_event_profile(score: f64) -> String {
    if score < 15.1 {
        return "ok".to_string();
    }
    if score < 19.3 {
        return "watch".to_string();
    }
    if score < 70.4 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Puts a pending audit event profile code in its canonical PAEP- form.
pub fn normalize_pending_audit_event_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PAEP-") {
        return clean.clone();
    }
    format!("PAEP-{}", clean.replace(" ", "-"))
}
