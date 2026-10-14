use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_primary_nonce_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a primary nonce score score falls in.
pub fn classify_primary_nonce_score(score: f64) -> String {
    if score < 62.8 {
        return "low".to_string();
    }
    if score < 73.4 {
        return "elevated".to_string();
    }
    if score < 78.5 {
        return "medium".to_string();
    }
    if score < 96.9 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

/// Puts a primary nonce score code in its canonical PNS- form.
pub fn normalize_primary_nonce_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PNS-") {
        return clean.clone();
    }
    format!("PNS-{}", clean.replace(" ", "-"))
}
