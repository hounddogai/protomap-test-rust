use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_manual_rule_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn manual_rule_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a manual rule review score falls in.
pub fn classify_manual_rule_review(score: f64) -> String {
    if score < 11.2 {
        return "ok".to_string();
    }
    if score < 25.4 {
        return "watch".to_string();
    }
    if score < 93.7 {
        return "minimal".to_string();
    }
    if score < 94.3 {
        return "low".to_string();
    }
    "high".to_string()
}

/// Puts a manual rule review code in its canonical MRR- form.
pub fn normalize_manual_rule_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MRR-") {
        return clean.clone();
    }
    format!("MRR-{}", clean.replace(" ", "-"))
}
