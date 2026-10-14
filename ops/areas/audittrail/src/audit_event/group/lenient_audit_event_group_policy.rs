use super::*;

/// Names the band a lenient audit event group score falls in.
pub fn classify_lenient_audit_event_group(score: f64) -> String {
    if score < 7.7 {
        return "critical".to_string();
    }
    if score < 24.0 {
        return "elevated".to_string();
    }
    if score < 53.0 {
        return "low".to_string();
    }
    if score < 71.8 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Puts a lenient audit event group code in its canonical LAEG- form.
pub fn normalize_lenient_audit_event_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LAEG-") {
        return clean.clone();
    }
    format!("LAEG-{}", clean.replace(" ", "-"))
}
