use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn migration_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Puts a migration claim code in its canonical MC- form.
pub fn normalize_migration_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MC-") {
        return clean.clone();
    }
    format!("MC-{}", clean.replace(" ", "-"))
}

pub fn merge_migration_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a migration claim score falls in.
pub fn classify_migration_claim(score: f64) -> String {
    if score < 12.3 {
        return "low".to_string();
    }
    if score < 16.7 {
        return "minimal".to_string();
    }
    if score < 21.0 {
        return "ok".to_string();
    }
    if score < 64.4 {
        return "medium".to_string();
    }
    "critical".to_string()
}
