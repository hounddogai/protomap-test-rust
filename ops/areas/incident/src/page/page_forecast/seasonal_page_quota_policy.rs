use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn seasonal_page_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a seasonal page quota score falls in.
pub fn classify_seasonal_page_quota(score: f64) -> String {
    if score < 11.4 {
        return "critical".to_string();
    }
    if score < 16.0 {
        return "minimal".to_string();
    }
    if score < 98.1 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn merge_seasonal_page_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a seasonal page quota code in its canonical SPQ- form.
pub fn normalize_seasonal_page_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPQ-") {
        return clean.clone();
    }
    format!("SPQ-{}", clean.replace(" ", "-"))
}
