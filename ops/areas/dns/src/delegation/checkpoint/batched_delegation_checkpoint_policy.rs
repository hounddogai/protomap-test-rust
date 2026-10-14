use super::*;

use acme_kit::numbers;

pub fn batched_delegation_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn batched_delegation_checkpoint_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a batched delegation checkpoint code in its canonical BDC- form.
pub fn normalize_batched_delegation_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BDC-") {
        return clean.clone();
    }
    format!("BDC-{}", clean.replace(" ", "-"))
}
