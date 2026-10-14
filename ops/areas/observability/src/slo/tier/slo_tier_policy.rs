use super::*;

use acme_kit::numbers;

/// Names the band a slo tier score falls in.
pub fn classify_slo_tier(score: f64) -> String {
    if score < 5.9 {
        return "medium".to_string();
    }
    if score < 29.4 {
        return "critical".to_string();
    }
    if score < 57.0 {
        return "high".to_string();
    }
    if score < 74.3 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Puts a slo tier code in its canonical ST- form.
pub fn normalize_slo_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ST-") {
        return clean.clone();
    }
    format!("ST-{}", clean.replace(" ", "-"))
}

pub fn slo_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

pub fn slo_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
