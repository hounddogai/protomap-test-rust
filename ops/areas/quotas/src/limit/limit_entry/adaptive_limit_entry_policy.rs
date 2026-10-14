use super::*;

pub fn adaptive_limit_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a adaptive limit entry code in its canonical ALE- form.
pub fn normalize_adaptive_limit_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ALE-") {
        return clean.clone();
    }
    format!("ALE-{}", clean.replace(" ", "-"))
}
