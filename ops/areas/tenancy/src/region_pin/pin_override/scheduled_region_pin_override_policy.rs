use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a scheduled region pin override code in its canonical SRPO- form.
pub fn normalize_scheduled_region_pin_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRPO-") {
        return clean.clone();
    }
    format!("SRPO-{}", clean.replace(" ", "-"))
}

pub fn merge_scheduled_region_pin_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled region pin override score falls in.
pub fn classify_scheduled_region_pin_override(score: f64) -> String {
    if score < 43.2 {
        return "critical".to_string();
    }
    if score < 70.7 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn scheduled_region_pin_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
