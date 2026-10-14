use super::*;

/// Names the band a batched release entry score falls in.
pub fn classify_batched_release_entry(score: f64) -> String {
    if score < 37.4 {
        return "high".to_string();
    }
    if score < 41.8 {
        return "medium".to_string();
    }
    if score < 97.6 {
        return "ok".to_string();
    }
    "critical".to_string()
}

/// Puts a batched release entry code in its canonical BRE- form.
pub fn normalize_batched_release_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BRE-") {
        return clean.clone();
    }
    format!("BRE-{}", clean.replace(" ", "-"))
}

pub fn batched_release_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
