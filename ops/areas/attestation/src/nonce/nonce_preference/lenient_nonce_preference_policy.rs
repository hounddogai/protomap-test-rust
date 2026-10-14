use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a lenient nonce preference code in its canonical LNP- form.
pub fn normalize_lenient_nonce_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LNP-") {
        return clean.clone();
    }
    format!("LNP-{}", clean.replace(" ", "-"))
}

pub fn merge_lenient_nonce_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
