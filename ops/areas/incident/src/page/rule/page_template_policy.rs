use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a page template score falls in.
pub fn classify_page_template(score: f64) -> String {
    if score < 32.3 {
        return "minimal".to_string();
    }
    if score < 60.7 {
        return "low".to_string();
    }
    if score < 86.2 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_page_template() -> i64 {
    let policy = retry::default_policy("page-template");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn merge_page_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn page_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.76;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
