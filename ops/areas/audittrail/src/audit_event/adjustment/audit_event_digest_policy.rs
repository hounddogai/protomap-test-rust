use super::*;

/// Puts a audit event digest code in its canonical AED- form.
pub fn normalize_audit_event_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AED-") {
        return clean.clone();
    }
    format!("AED-{}", clean.replace(" ", "-"))
}

pub fn audit_event_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
