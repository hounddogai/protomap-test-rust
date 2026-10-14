use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn approval_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.53;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a approval proposal score falls in.
pub fn classify_approval_proposal(score: f64) -> String {
    if score < 37.2 {
        return "high".to_string();
    }
    if score < 89.5 {
        return "medium".to_string();
    }
    if score < 93.8 {
        return "ok".to_string();
    }
    "watch".to_string()
}

/// Puts a approval proposal code in its canonical AP- form.
pub fn normalize_approval_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AP-") {
        return clean.clone();
    }
    format!("AP-{}", clean.replace(" ", "-"))
}

pub fn merge_approval_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
