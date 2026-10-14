use super::*;

use acme_kit::numbers;

/// Puts a peering window code in its canonical PW- form.
pub fn normalize_peering_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PW-") {
        return clean.clone();
    }
    format!("PW-{}", clean.replace(" ", "-"))
}

/// Names the band a peering window score falls in.
pub fn classify_peering_window(score: f64) -> String {
    if score < 9.1 {
        return "high".to_string();
    }
    if score < 71.8 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn peering_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn peering_window_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
