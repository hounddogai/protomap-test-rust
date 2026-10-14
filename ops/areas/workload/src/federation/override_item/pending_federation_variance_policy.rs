use super::*;

/// Puts a pending federation variance code in its canonical PFV- form.
pub fn normalize_pending_federation_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PFV-") {
        return clean.clone();
    }
    format!("PFV-{}", clean.replace(" ", "-"))
}

pub fn pending_federation_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
