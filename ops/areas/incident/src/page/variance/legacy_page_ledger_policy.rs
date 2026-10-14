use super::*;

use acme_kit::numbers;

pub fn legacy_page_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a legacy page ledger code in its canonical LPL- form.
pub fn normalize_legacy_page_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LPL-") {
        return clean.clone();
    }
    format!("LPL-{}", clean.replace(" ", "-"))
}

pub fn legacy_page_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
