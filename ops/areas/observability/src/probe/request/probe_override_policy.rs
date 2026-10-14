use super::*;

pub fn probe_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Puts a probe override code in its canonical PO- form.
pub fn normalize_probe_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PO-") {
        return clean.clone();
    }
    format!("PO-{}", clean.replace(" ", "-"))
}
