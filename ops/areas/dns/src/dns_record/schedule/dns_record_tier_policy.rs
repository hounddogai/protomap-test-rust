use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_dns_record_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a dns record tier code in its canonical DRT- form.
pub fn normalize_dns_record_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRT-") {
        return clean.clone();
    }
    format!("DRT-{}", clean.replace(" ", "-"))
}
