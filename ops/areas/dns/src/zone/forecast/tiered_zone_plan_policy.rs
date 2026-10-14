use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_tiered_zone_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a tiered zone plan score falls in.
pub fn classify_tiered_zone_plan(score: f64) -> String {
    if score < 17.0 {
        return "high".to_string();
    }
    if score < 32.9 {
        return "critical".to_string();
    }
    if score < 33.2 {
        return "watch".to_string();
    }
    if score < 37.7 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn tiered_zone_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
