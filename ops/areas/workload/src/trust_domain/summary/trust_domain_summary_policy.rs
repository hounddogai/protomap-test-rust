use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_trust_domain_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a trust domain summary score falls in.
pub fn classify_trust_domain_summary(score: f64) -> String {
    if score < 48.1 {
        return "ok".to_string();
    }
    if score < 62.8 {
        return "watch".to_string();
    }
    if score < 67.3 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Puts a trust domain summary code in its canonical TDS- form.
pub fn normalize_trust_domain_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDS-") {
        return clean.clone();
    }
    format!("TDS-{}", clean.replace(" ", "-"))
}

pub fn trust_domain_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
