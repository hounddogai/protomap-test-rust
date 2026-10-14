use super::*;

use acme_kit::numbers;

pub fn token_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a token notice code in its canonical TN- form.
pub fn normalize_token_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TN-") {
        return clean.clone();
    }
    format!("TN-{}", clean.replace(" ", "-"))
}

pub fn token_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a token notice score falls in.
pub fn classify_token_notice(score: f64) -> String {
    if score < 10.9 {
        return "ok".to_string();
    }
    if score < 58.6 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
