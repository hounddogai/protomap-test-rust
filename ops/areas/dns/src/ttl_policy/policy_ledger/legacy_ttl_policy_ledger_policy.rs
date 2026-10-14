use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn legacy_ttl_policy_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Puts a legacy ttl policy ledger code in its canonical LTPL- form.
pub fn normalize_legacy_ttl_policy_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LTPL-") {
        return clean.clone();
    }
    format!("LTPL-{}", clean.replace(" ", "-"))
}

pub fn merge_legacy_ttl_policy_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn legacy_ttl_policy_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
