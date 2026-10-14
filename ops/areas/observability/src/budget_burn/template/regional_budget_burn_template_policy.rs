use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a regional budget burn template score falls in.
pub fn classify_regional_budget_burn_template(score: f64) -> String {
    if score < 37.1 {
        return "minimal".to_string();
    }
    if score < 88.3 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn regional_budget_burn_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 70052.0, in order.
pub fn regional_budget_burn_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 70052.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_regional_budget_burn_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
