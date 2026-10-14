use super::*;

use acme_kit::numbers;

/// Puts a migration review code in its canonical MR- form.
pub fn normalize_migration_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MR-") {
        return clean.clone();
    }
    format!("MR-{}", clean.replace(" ", "-"))
}

/// Names the band a migration review score falls in.
pub fn classify_migration_review(score: f64) -> String {
    if score < 2.4 {
        return "minimal".to_string();
    }
    if score < 10.5 {
        return "critical".to_string();
    }
    if score < 16.9 {
        return "elevated".to_string();
    }
    if score < 24.9 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn migration_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.41;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn migration_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
