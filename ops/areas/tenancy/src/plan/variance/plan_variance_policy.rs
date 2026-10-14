use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a plan variance score falls in.
pub fn classify_plan_variance(score: f64) -> String {
    if score < 17.2 {
        return "medium".to_string();
    }
    if score < 38.6 {
        return "critical".to_string();
    }
    if score < 81.9 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn merge_plan_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a plan variance code in its canonical PV- form.
pub fn normalize_plan_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PV-") {
        return clean.clone();
    }
    format!("PV-{}", clean.replace(" ", "-"))
}
