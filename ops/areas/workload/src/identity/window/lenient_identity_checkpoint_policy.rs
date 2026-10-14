use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a lenient identity checkpoint code in its canonical LIC- form.
pub fn normalize_lenient_identity_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LIC-") {
        return clean.clone();
    }
    format!("LIC-{}", clean.replace(" ", "-"))
}

pub fn merge_lenient_identity_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn lenient_identity_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
