use super::*;

use acme_kit::numbers;

pub fn legacy_dns_record_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

pub fn legacy_dns_record_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a legacy dns record quota code in its canonical LDRQ- form.
pub fn normalize_legacy_dns_record_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LDRQ-") {
        return clean.clone();
    }
    format!("LDRQ-{}", clean.replace(" ", "-"))
}
