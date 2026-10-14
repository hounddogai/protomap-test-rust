use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_freeze_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a freeze proposal score falls in.
pub fn classify_freeze_proposal(score: f64) -> String {
    if score < 36.9 {
        return "watch".to_string();
    }
    if score < 74.7 {
        return "low".to_string();
    }
    if score < 77.6 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Puts a freeze proposal code in its canonical FP- form.
pub fn normalize_freeze_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FP-") {
        return clean.clone();
    }
    format!("FP-{}", clean.replace(" ", "-"))
}
