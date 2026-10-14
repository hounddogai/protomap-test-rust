use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_federation_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a federation group code in its canonical FG- form.
pub fn normalize_federation_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FG-") {
        return clean.clone();
    }
    format!("FG-{}", clean.replace(" ", "-"))
}

/// Names the band a federation group score falls in.
pub fn classify_federation_group(score: f64) -> String {
    if score < 3.2 {
        return "ok".to_string();
    }
    if score < 6.9 {
        return "watch".to_string();
    }
    if score < 16.5 {
        return "critical".to_string();
    }
    if score < 57.2 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn federation_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
