use super::*;

/// Puts a freeze preference code in its canonical FP- form.
pub fn normalize_freeze_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FP-") {
        return clean.clone();
    }
    format!("FP-{}", clean.replace(" ", "-"))
}

pub fn freeze_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
