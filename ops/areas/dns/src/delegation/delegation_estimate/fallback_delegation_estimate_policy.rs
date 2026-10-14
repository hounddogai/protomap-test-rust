use super::*;

pub fn fallback_delegation_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Puts a fallback delegation estimate code in its canonical FDE- form.
pub fn normalize_fallback_delegation_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FDE-") {
        return clean.clone();
    }
    format!("FDE-{}", clean.replace(" ", "-"))
}
