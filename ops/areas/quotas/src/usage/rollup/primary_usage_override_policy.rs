use super::*;

pub fn primary_usage_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Puts a primary usage override code in its canonical PUO- form.
pub fn normalize_primary_usage_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PUO-") {
        return clean.clone();
    }
    format!("PUO-{}", clean.replace(" ", "-"))
}
