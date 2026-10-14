use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_tiered_migration_milestone_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a tiered migration milestone code in its canonical TMM- form.
pub fn normalize_tiered_migration_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TMM-") {
        return clean.clone();
    }
    format!("TMM-{}", clean.replace(" ", "-"))
}
