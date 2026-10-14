use super::*;

pub fn weighted_page_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a weighted page adjustment code in its canonical WPA- form.
pub fn normalize_weighted_page_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WPA-") {
        return clean.clone();
    }
    format!("WPA-{}", clean.replace(" ", "-"))
}
