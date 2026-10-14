use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a manual suspension quota score falls in.
pub fn classify_manual_suspension_quota(score: f64) -> String {
    if score < 2.6 {
        return "ok".to_string();
    }
    if score < 11.7 {
        return "minimal".to_string();
    }
    if score < 31.0 {
        return "low".to_string();
    }
    if score < 38.4 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn manual_suspension_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_manual_suspension_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a manual suspension quota code in its canonical MSQ- form.
pub fn normalize_manual_suspension_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MSQ-") {
        return clean.clone();
    }
    format!("MSQ-{}", clean.replace(" ", "-"))
}
