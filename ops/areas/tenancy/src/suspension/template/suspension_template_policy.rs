use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a suspension template score falls in.
pub fn classify_suspension_template(score: f64) -> String {
    if score < 8.0 {
        return "high".to_string();
    }
    if score < 92.0 {
        return "ok".to_string();
    }
    if score < 97.4 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn merge_suspension_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
