use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_trust_domain_allocation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a trust domain allocation score falls in.
pub fn classify_trust_domain_allocation(score: f64) -> String {
    if score < 2.8 {
        return "minimal".to_string();
    }
    if score < 11.0 {
        return "low".to_string();
    }
    if score < 38.5 {
        return "watch".to_string();
    }
    if score < 87.7 {
        return "medium".to_string();
    }
    "critical".to_string()
}
