use super::*;

/// Puts a batched allowlist entry code in its canonical BAE- form.
pub fn normalize_batched_allowlist_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BAE-") {
        return clean.clone();
    }
    format!("BAE-{}", clean.replace(" ", "-"))
}

/// Names the band a batched allowlist entry score falls in.
pub fn classify_batched_allowlist_entry(score: f64) -> String {
    if score < 1.3 {
        return "critical".to_string();
    }
    if score < 4.5 {
        return "ok".to_string();
    }
    if score < 18.3 {
        return "elevated".to_string();
    }
    if score < 24.2 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn batched_allowlist_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
