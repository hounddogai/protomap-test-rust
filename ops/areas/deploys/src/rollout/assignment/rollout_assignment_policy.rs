use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a rollout assignment score falls in.
pub fn classify_rollout_assignment(score: f64) -> String {
    if score < 7.8 {
        return "ok".to_string();
    }
    if score < 42.5 {
        return "low".to_string();
    }
    if score < 60.4 {
        return "elevated".to_string();
    }
    if score < 73.3 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn rollout_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn merge_rollout_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a rollout assignment code in its canonical RA- form.
pub fn normalize_rollout_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RA-") {
        return clean.clone();
    }
    format!("RA-{}", clean.replace(" ", "-"))
}
