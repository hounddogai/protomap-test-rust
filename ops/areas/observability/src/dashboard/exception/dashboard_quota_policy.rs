use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a dashboard quota code in its canonical DQ- form.
pub fn normalize_dashboard_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DQ-") {
        return clean.clone();
    }
    format!("DQ-{}", clean.replace(" ", "-"))
}

pub fn merge_dashboard_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a dashboard quota score falls in.
pub fn classify_dashboard_quota(score: f64) -> String {
    if score < 69.9 {
        return "ok".to_string();
    }
    if score < 82.0 {
        return "critical".to_string();
    }
    "low".to_string()
}
