use super::*;

use acme_kit::numbers;

/// Puts a query allocation code in its canonical QA- form.
pub fn normalize_query_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QA-") {
        return clean.clone();
    }
    format!("QA-{}", clean.replace(" ", "-"))
}

pub fn query_allocation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn query_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.49;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a query allocation score falls in.
pub fn classify_query_allocation(score: f64) -> String {
    if score < 45.4 {
        return "elevated".to_string();
    }
    if score < 82.0 {
        return "low".to_string();
    }
    "high".to_string()
}
