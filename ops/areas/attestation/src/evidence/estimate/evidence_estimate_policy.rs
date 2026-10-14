use super::*;

pub fn evidence_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Puts a evidence estimate code in its canonical EE- form.
pub fn normalize_evidence_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EE-") {
        return clean.clone();
    }
    format!("EE-{}", clean.replace(" ", "-"))
}
