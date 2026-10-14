use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a primary suspension score score falls in.
pub fn classify_primary_suspension_score(score: f64) -> String {
    if score < 39.9 {
        return "minimal".to_string();
    }
    if score < 77.2 {
        return "medium".to_string();
    }
    if score < 87.0 {
        return "critical".to_string();
    }
    if score < 90.2 {
        return "high".to_string();
    }
    "watch".to_string()
}

/// Puts a primary suspension score code in its canonical PSS- form.
pub fn normalize_primary_suspension_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PSS-") {
        return clean.clone();
    }
    format!("PSS-{}", clean.replace(" ", "-"))
}

pub fn merge_primary_suspension_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
