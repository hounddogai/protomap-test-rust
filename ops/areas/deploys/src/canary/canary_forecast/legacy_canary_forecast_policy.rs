use super::*;

use acme_kit::numbers;

/// Puts a legacy canary forecast code in its canonical LCF- form.
pub fn normalize_legacy_canary_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LCF-") {
        return clean.clone();
    }
    format!("LCF-{}", clean.replace(" ", "-"))
}

pub fn legacy_canary_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn legacy_canary_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
