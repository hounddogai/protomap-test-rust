use super::*;

use acme_kit::numbers;

/// Puts a rolling release record code in its canonical RRR- form.
pub fn normalize_rolling_release_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRR-") {
        return clean.clone();
    }
    format!("RRR-{}", clean.replace(" ", "-"))
}

pub fn rolling_release_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

pub fn rolling_release_record_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
