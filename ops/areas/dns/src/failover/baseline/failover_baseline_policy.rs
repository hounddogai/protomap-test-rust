use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn failover_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn merge_failover_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a failover baseline score falls in.
pub fn classify_failover_baseline(score: f64) -> String {
    if score < 43.2 {
        return "ok".to_string();
    }
    if score < 90.5 {
        return "medium".to_string();
    }
    "elevated".to_string()
}
