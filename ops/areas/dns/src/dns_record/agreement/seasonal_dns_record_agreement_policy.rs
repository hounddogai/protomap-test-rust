use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a seasonal dns record agreement score falls in.
pub fn classify_seasonal_dns_record_agreement(score: f64) -> String {
    if score < 76.8 {
        return "minimal".to_string();
    }
    if score < 99.3 {
        return "high".to_string();
    }
    "critical".to_string()
}

/// Puts a seasonal dns record agreement code in its canonical SDRA- form.
pub fn normalize_seasonal_dns_record_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SDRA-") {
        return clean.clone();
    }
    format!("SDRA-{}", clean.replace(" ", "-"))
}

pub fn seasonal_dns_record_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

pub fn merge_seasonal_dns_record_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
