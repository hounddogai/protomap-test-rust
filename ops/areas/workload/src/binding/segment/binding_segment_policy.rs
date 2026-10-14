use super::*;

/// Puts a binding segment code in its canonical BS- form.
pub fn normalize_binding_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BS-") {
        return clean.clone();
    }
    format!("BS-{}", clean.replace(" ", "-"))
}

pub fn binding_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
