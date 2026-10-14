use super::*;

use acme_kit::numbers;

pub fn primary_zone_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a primary zone metric code in its canonical PZM- form.
pub fn normalize_primary_zone_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PZM-") {
        return clean.clone();
    }
    format!("PZM-{}", clean.replace(" ", "-"))
}

pub fn primary_zone_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
