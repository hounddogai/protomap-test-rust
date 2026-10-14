use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_default_dns_record_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a default dns record segment code in its canonical DDRS- form.
pub fn normalize_default_dns_record_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DDRS-") {
        return clean.clone();
    }
    format!("DDRS-{}", clean.replace(" ", "-"))
}

pub fn default_dns_record_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Names the band a default dns record segment score falls in.
pub fn classify_default_dns_record_segment(score: f64) -> String {
    if score < 49.3 {
        return "elevated".to_string();
    }
    if score < 49.5 {
        return "minimal".to_string();
    }
    if score < 61.1 {
        return "low".to_string();
    }
    "critical".to_string()
}
