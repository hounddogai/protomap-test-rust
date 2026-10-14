use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn lenient_slo_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 81060.0, in order.
pub fn lenient_slo_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 81060.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a lenient slo draft score falls in.
pub fn classify_lenient_slo_draft(score: f64) -> String {
    if score < 12.7 {
        return "critical".to_string();
    }
    if score < 19.1 {
        return "high".to_string();
    }
    if score < 38.5 {
        return "low".to_string();
    }
    if score < 72.8 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn merge_lenient_slo_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
