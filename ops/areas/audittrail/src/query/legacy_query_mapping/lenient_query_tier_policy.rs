use super::*;

use acme_kit::numbers;

pub fn lenient_query_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn lenient_query_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Puts a lenient query tier code in its canonical LQT- form.
pub fn normalize_lenient_query_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LQT-") {
        return clean.clone();
    }
    format!("LQT-{}", clean.replace(" ", "-"))
}
