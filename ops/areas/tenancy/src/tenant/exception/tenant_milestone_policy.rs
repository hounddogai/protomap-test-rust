use super::*;

/// Puts a tenant milestone code in its canonical TM- form.
pub fn normalize_tenant_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TM-") {
        return clean.clone();
    }
    format!("TM-{}", clean.replace(" ", "-"))
}

pub fn tenant_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
