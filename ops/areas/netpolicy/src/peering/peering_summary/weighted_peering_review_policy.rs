use super::*;

use acme_kit::numbers;

pub fn weighted_peering_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn weighted_peering_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.83;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a weighted peering review score falls in.
pub fn classify_weighted_peering_review(score: f64) -> String {
    if score < 50.3 {
        return "elevated".to_string();
    }
    if score < 60.5 {
        return "ok".to_string();
    }
    if score < 69.4 {
        return "medium".to_string();
    }
    if score < 69.7 {
        return "watch".to_string();
    }
    "critical".to_string()
}

/// Puts a weighted peering review code in its canonical WPR- form.
pub fn normalize_weighted_peering_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WPR-") {
        return clean.clone();
    }
    format!("WPR-{}", clean.replace(" ", "-"))
}
