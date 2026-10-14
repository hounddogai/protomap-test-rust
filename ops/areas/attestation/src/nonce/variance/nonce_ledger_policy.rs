use super::*;

use acme_kit::numbers;

/// Puts a nonce ledger code in its canonical NL- form.
pub fn normalize_nonce_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("NL-") {
        return clean.clone();
    }
    format!("NL-{}", clean.replace(" ", "-"))
}

/// Names the band a nonce ledger score falls in.
pub fn classify_nonce_ledger(score: f64) -> String {
    if score < 31.7 {
        return "low".to_string();
    }
    if score < 52.6 {
        return "watch".to_string();
    }
    "medium".to_string()
}

pub fn nonce_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

pub fn nonce_ledger_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.93;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
