use super::*;

/// Puts a strict silence reservation code in its canonical SSR- form.
pub fn normalize_strict_silence_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SSR-") {
        return clean.clone();
    }
    format!("SSR-{}", clean.replace(" ", "-"))
}

pub fn strict_silence_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
