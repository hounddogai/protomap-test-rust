use super::*;

/// Puts a lenient action item batch code in its canonical LAIB- form.
pub fn normalize_lenient_action_item_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LAIB-") {
        return clean.clone();
    }
    format!("LAIB-{}", clean.replace(" ", "-"))
}

pub fn lenient_action_item_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
