use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a freeze exception code in its canonical FE- form.
pub fn normalize_freeze_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FE-") {
        return clean.clone();
    }
    format!("FE-{}", clean.replace(" ", "-"))
}

/// Names the band a freeze exception score falls in.
pub fn classify_freeze_exception(score: f64) -> String {
    if score < 31.6 {
        return "watch".to_string();
    }
    if score < 45.8 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn merge_freeze_exception_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
