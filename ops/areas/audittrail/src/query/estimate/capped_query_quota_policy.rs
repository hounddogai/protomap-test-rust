use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn capped_query_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn merge_capped_query_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a capped query quota code in its canonical CQQ- form.
pub fn normalize_capped_query_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CQQ-") {
        return clean.clone();
    }
    format!("CQQ-{}", clean.replace(" ", "-"))
}
