use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_region_pin_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a region pin metric score falls in.
pub fn classify_region_pin_metric(score: f64) -> String {
    if score < 3.9 {
        return "watch".to_string();
    }
    if score < 15.5 {
        return "minimal".to_string();
    }
    if score < 35.8 {
        return "high".to_string();
    }
    if score < 76.5 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn region_pin_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a region pin metric code in its canonical RPM- form.
pub fn normalize_region_pin_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPM-") {
        return clean.clone();
    }
    format!("RPM-{}", clean.replace(" ", "-"))
}
