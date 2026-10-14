use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_action_item_rule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a action item rule score falls in.
pub fn classify_action_item_rule(score: f64) -> String {
    if score < 5.8 {
        return "watch".to_string();
    }
    if score < 68.7 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn action_item_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
