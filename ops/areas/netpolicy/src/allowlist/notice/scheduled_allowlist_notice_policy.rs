use super::*;

/// Puts a scheduled allowlist notice code in its canonical SAN- form.
pub fn normalize_scheduled_allowlist_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAN-") {
        return clean.clone();
    }
    format!("SAN-{}", clean.replace(" ", "-"))
}

pub fn scheduled_allowlist_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}
