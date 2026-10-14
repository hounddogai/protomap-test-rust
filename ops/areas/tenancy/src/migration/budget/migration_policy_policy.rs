use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a migration policy code in its canonical MP- form.
pub fn normalize_migration_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MP-") {
        return clean.clone();
    }
    format!("MP-{}", clean.replace(" ", "-"))
}

pub fn merge_migration_policy_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a migration policy score falls in.
pub fn classify_migration_policy(score: f64) -> String {
    if score < 63.0 {
        return "critical".to_string();
    }
    if score < 86.6 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
