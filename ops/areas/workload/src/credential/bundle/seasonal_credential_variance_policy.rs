use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn seasonal_credential_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.22;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn seasonal_credential_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_seasonal_credential_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a seasonal credential variance score falls in.
pub fn classify_seasonal_credential_variance(score: f64) -> String {
    if score < 34.3 {
        return "critical".to_string();
    }
    if score < 45.7 {
        return "low".to_string();
    }
    if score < 79.5 {
        return "watch".to_string();
    }
    "ok".to_string()
}
