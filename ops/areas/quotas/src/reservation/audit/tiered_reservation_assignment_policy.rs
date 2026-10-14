use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a tiered reservation assignment code in its canonical TRA- form.
pub fn normalize_tiered_reservation_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TRA-") {
        return clean.clone();
    }
    format!("TRA-{}", clean.replace(" ", "-"))
}

pub fn merge_tiered_reservation_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
