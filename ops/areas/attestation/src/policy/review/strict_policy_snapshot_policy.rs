use super::*;

pub fn strict_policy_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a strict policy snapshot code in its canonical SPS- form.
pub fn normalize_strict_policy_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPS-") {
        return clean.clone();
    }
    format!("SPS-{}", clean.replace(" ", "-"))
}
