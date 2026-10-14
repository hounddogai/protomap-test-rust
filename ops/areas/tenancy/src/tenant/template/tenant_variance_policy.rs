use super::*;

/// Puts a tenant variance code in its canonical TV- form.
pub fn normalize_tenant_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TV-") {
        return clean.clone();
    }
    format!("TV-{}", clean.replace(" ", "-"))
}

pub fn tenant_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
