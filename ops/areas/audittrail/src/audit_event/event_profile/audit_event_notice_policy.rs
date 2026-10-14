use super::*;

use acme_kit::numbers;

pub fn audit_event_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a audit event notice code in its canonical AEN- form.
pub fn normalize_audit_event_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AEN-") {
        return clean.clone();
    }
    format!("AEN-{}", clean.replace(" ", "-"))
}

pub fn audit_event_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
