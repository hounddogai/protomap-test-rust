use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_tenant_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a tenant plan score falls in.
pub fn classify_tenant_plan(score: f64) -> String {
    if score < 4.2 {
        return "low".to_string();
    }
    if score < 19.2 {
        return "critical".to_string();
    }
    if score < 34.8 {
        return "watch".to_string();
    }
    if score < 38.4 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn tenant_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
