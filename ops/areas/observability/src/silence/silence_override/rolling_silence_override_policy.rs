use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a rolling silence override code in its canonical RSO- form.
pub fn normalize_rolling_silence_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RSO-") {
        return clean.clone();
    }
    format!("RSO-{}", clean.replace(" ", "-"))
}

pub fn merge_rolling_silence_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a rolling silence override score falls in.
pub fn classify_rolling_silence_override(score: f64) -> String {
    if score < 6.8 {
        return "medium".to_string();
    }
    if score < 21.4 {
        return "minimal".to_string();
    }
    if score < 39.4 {
        return "ok".to_string();
    }
    if score < 53.0 {
        return "elevated".to_string();
    }
    "low".to_string()
}
