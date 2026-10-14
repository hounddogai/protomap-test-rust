use super::*;

use acme_kit::numbers;

/// Puts a dns record review code in its canonical DRR- form.
pub fn normalize_dns_record_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRR-") {
        return clean.clone();
    }
    format!("DRR-{}", clean.replace(" ", "-"))
}

pub fn dns_record_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn dns_record_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn dns_record_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.97;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
