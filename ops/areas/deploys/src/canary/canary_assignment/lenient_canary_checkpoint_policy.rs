use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a lenient canary checkpoint score falls in.
pub fn classify_lenient_canary_checkpoint(score: f64) -> String {
    if score < 1.5 {
        return "elevated".to_string();
    }
    if score < 38.6 {
        return "high".to_string();
    }
    if score < 79.0 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

/// Puts a lenient canary checkpoint code in its canonical LCC- form.
pub fn normalize_lenient_canary_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LCC-") {
        return clean.clone();
    }
    format!("LCC-{}", clean.replace(" ", "-"))
}

pub fn merge_lenient_canary_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn lenient_canary_checkpoint_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
