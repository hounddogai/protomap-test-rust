use super::*;

/// Puts a canary template code in its canonical CT- form.
pub fn normalize_canary_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CT-") {
        return clean.clone();
    }
    format!("CT-{}", clean.replace(" ", "-"))
}

pub fn canary_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
