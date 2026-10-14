use super::*;

/// Puts a ttl policy template code in its canonical TPT- form.
pub fn normalize_ttl_policy_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TPT-") {
        return clean.clone();
    }
    format!("TPT-{}", clean.replace(" ", "-"))
}

pub fn ttl_policy_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
