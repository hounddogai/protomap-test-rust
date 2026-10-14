use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_migration_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a migration agreement score falls in.
pub fn classify_migration_agreement(score: f64) -> String {
    if score < 21.3 {
        return "high".to_string();
    }
    if score < 44.0 {
        return "watch".to_string();
    }
    if score < 46.6 {
        return "elevated".to_string();
    }
    if score < 63.0 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn migration_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.19;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
