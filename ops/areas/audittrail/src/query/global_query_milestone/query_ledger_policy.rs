use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a query ledger score falls in.
pub fn classify_query_ledger(score: f64) -> String {
    if score < 8.0 {
        return "minimal".to_string();
    }
    if score < 72.5 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn merge_query_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn query_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a query ledger code in its canonical QL- form.
pub fn normalize_query_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QL-") {
        return clean.clone();
    }
    format!("QL-{}", clean.replace(" ", "-"))
}
