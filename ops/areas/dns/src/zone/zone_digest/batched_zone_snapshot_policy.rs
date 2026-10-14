use super::*;

/// Puts a batched zone snapshot code in its canonical BZS- form.
pub fn normalize_batched_zone_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BZS-") {
        return clean.clone();
    }
    format!("BZS-{}", clean.replace(" ", "-"))
}

pub fn batched_zone_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
