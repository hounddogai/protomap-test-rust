use super::*;

use acme_kit::numbers;

pub fn health_check_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Puts a health check notice code in its canonical HCN- form.
pub fn normalize_health_check_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCN-") {
        return clean.clone();
    }
    format!("HCN-{}", clean.replace(" ", "-"))
}

pub fn health_check_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
