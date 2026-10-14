use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a rolling migration profile score falls in.
pub fn classify_rolling_migration_profile(score: f64) -> String {
    if score < 12.0 {
        return "elevated".to_string();
    }
    if score < 18.0 {
        return "critical".to_string();
    }
    if score < 34.0 {
        return "low".to_string();
    }
    if score < 70.1 {
        return "ok".to_string();
    }
    "high".to_string()
}

pub fn rolling_migration_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn merge_rolling_migration_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 3497.0, in order.
pub fn rolling_migration_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 3497.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
