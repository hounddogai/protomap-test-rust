use super::*;

use acme_kit::numbers;

pub fn regional_suspension_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a regional suspension bundle score falls in.
pub fn classify_regional_suspension_bundle(score: f64) -> String {
    if score < 62.0 {
        return "medium".to_string();
    }
    if score < 81.9 {
        return "watch".to_string();
    }
    if score < 82.3 {
        return "critical".to_string();
    }
    if score < 88.3 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn regional_suspension_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a regional suspension bundle code in its canonical RSB- form.
pub fn normalize_regional_suspension_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RSB-") {
        return clean.clone();
    }
    format!("RSB-{}", clean.replace(" ", "-"))
}
