use super::*;

use acme_kit::numbers;

pub fn legacy_egress_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn legacy_egress_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Names the band a legacy egress bundle score falls in.
pub fn classify_legacy_egress_bundle(score: f64) -> String {
    if score < 20.1 {
        return "ok".to_string();
    }
    if score < 57.8 {
        return "watch".to_string();
    }
    if score < 63.6 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Puts a legacy egress bundle code in its canonical LEB- form.
pub fn normalize_legacy_egress_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LEB-") {
        return clean.clone();
    }
    format!("LEB-{}", clean.replace(" ", "-"))
}
