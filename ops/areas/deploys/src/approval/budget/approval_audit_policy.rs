use super::*;

/// Puts a approval audit code in its canonical AA- form.
pub fn normalize_approval_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AA-") {
        return clean.clone();
    }
    format!("AA-{}", clean.replace(" ", "-"))
}

pub fn approval_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
