use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn tiered_dns_record_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a tiered dns record entry score falls in.
pub fn classify_tiered_dns_record_entry(score: f64) -> String {
    if score < 57.6 {
        return "ok".to_string();
    }
    if score < 64.9 {
        return "elevated".to_string();
    }
    if score < 73.5 {
        return "minimal".to_string();
    }
    if score < 88.3 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn merge_tiered_dns_record_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a tiered dns record entry code in its canonical TDRE- form.
pub fn normalize_tiered_dns_record_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDRE-") {
        return clean.clone();
    }
    format!("TDRE-{}", clean.replace(" ", "-"))
}
