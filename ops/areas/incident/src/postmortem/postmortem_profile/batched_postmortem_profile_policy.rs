use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_batched_postmortem_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 84402.0, in order.
pub fn batched_postmortem_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84402.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched postmortem profile score falls in.
pub fn classify_batched_postmortem_profile(score: f64) -> String {
    if score < 23.3 {
        return "ok".to_string();
    }
    if score < 51.6 {
        return "minimal".to_string();
    }
    if score < 62.8 {
        return "high".to_string();
    }
    "low".to_string()
}
