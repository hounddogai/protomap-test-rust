use super::*;

/// Puts a page threshold code in its canonical PT- form.
pub fn normalize_page_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PT-") {
        return clean.clone();
    }
    format!("PT-{}", clean.replace(" ", "-"))
}

pub fn page_threshold_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
