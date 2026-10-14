use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_dns_record_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a dns record profile score falls in.
pub fn classify_dns_record_profile(score: f64) -> String {
    if score < 33.0 {
        return "high".to_string();
    }
    if score < 89.7 {
        return "elevated".to_string();
    }
    if score < 96.7 {
        return "ok".to_string();
    }
    "critical".to_string()
}
