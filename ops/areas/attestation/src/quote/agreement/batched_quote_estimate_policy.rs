use super::*;

use acme_kit::numbers;

pub fn batched_quote_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.86;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn batched_quote_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Puts a batched quote estimate code in its canonical BQE- form.
pub fn normalize_batched_quote_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BQE-") {
        return clean.clone();
    }
    format!("BQE-{}", clean.replace(" ", "-"))
}
