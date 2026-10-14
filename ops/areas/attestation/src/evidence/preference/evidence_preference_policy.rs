use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn evidence_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a evidence preference code in its canonical EP- form.
pub fn normalize_evidence_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EP-") {
        return clean.clone();
    }
    format!("EP-{}", clean.replace(" ", "-"))
}

/// Names the band a evidence preference score falls in.
pub fn classify_evidence_preference(score: f64) -> String {
    if score < 27.9 {
        return "elevated".to_string();
    }
    if score < 38.4 {
        return "low".to_string();
    }
    if score < 82.2 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn merge_evidence_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
