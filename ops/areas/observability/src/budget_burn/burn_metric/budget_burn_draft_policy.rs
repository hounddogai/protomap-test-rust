use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_budget_burn_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn budget_burn_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.88;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a budget burn draft score falls in.
pub fn classify_budget_burn_draft(score: f64) -> String {
    if score < 58.5 {
        return "high".to_string();
    }
    if score < 61.1 {
        return "medium".to_string();
    }
    if score < 80.9 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn budget_burn_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
