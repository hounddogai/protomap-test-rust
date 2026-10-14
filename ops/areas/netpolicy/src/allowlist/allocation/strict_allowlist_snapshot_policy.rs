use super::*;

/// Puts a strict allowlist snapshot code in its canonical SAS- form.
pub fn normalize_strict_allowlist_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAS-") {
        return clean.clone();
    }
    format!("SAS-{}", clean.replace(" ", "-"))
}

pub fn strict_allowlist_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
