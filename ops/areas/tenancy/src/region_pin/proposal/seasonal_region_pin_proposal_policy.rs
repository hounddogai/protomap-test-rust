use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a seasonal region pin proposal score falls in.
pub fn classify_seasonal_region_pin_proposal(score: f64) -> String {
    if score < 38.3 {
        return "medium".to_string();
    }
    if score < 97.1 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Puts a seasonal region pin proposal code in its canonical SRPP- form.
pub fn normalize_seasonal_region_pin_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRPP-") {
        return clean.clone();
    }
    format!("SRPP-{}", clean.replace(" ", "-"))
}

pub fn merge_seasonal_region_pin_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
