use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a primary page baseline score falls in.
pub fn classify_primary_page_baseline(score: f64) -> String {
    if score < 5.4 {
        return "minimal".to_string();
    }
    if score < 66.7 {
        return "low".to_string();
    }
    if score < 98.7 {
        return "elevated".to_string();
    }
    "high".to_string()
}

/// Puts a primary page baseline code in its canonical PPB- form.
pub fn normalize_primary_page_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PPB-") {
        return clean.clone();
    }
    format!("PPB-{}", clean.replace(" ", "-"))
}

pub fn merge_primary_page_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
