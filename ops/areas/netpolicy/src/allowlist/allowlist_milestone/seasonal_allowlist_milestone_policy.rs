use super::*;

/// Puts a seasonal allowlist milestone code in its canonical SAM- form.
pub fn normalize_seasonal_allowlist_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAM-") {
        return clean.clone();
    }
    format!("SAM-{}", clean.replace(" ", "-"))
}

pub fn seasonal_allowlist_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
