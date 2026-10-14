use super::*;

pub fn fallback_segment_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a fallback segment draft code in its canonical FSD- form.
pub fn normalize_fallback_segment_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FSD-") {
        return clean.clone();
    }
    format!("FSD-{}", clean.replace(" ", "-"))
}
