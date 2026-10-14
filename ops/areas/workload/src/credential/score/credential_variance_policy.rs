use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a credential variance score falls in.
pub fn classify_credential_variance(score: f64) -> String {
    if score < 3.8 {
        return "high".to_string();
    }
    if score < 21.7 {
        return "watch".to_string();
    }
    "medium".to_string()
}

pub fn credential_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a credential variance code in its canonical CV- form.
pub fn normalize_credential_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CV-") {
        return clean.clone();
    }
    format!("CV-{}", clean.replace(" ", "-"))
}

pub fn merge_credential_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
