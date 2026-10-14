use super::*;

use acme_kit::numbers;

pub fn primary_region_pin_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn primary_region_pin_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a primary region pin manifest code in its canonical PRPM- form.
pub fn normalize_primary_region_pin_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRPM-") {
        return clean.clone();
    }
    format!("PRPM-{}", clean.replace(" ", "-"))
}
