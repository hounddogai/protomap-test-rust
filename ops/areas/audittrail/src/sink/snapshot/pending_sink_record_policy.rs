use super::*;

pub fn pending_sink_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Puts a pending sink record code in its canonical PSR- form.
pub fn normalize_pending_sink_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PSR-") {
        return clean.clone();
    }
    format!("PSR-{}", clean.replace(" ", "-"))
}
