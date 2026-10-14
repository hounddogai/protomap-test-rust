use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_primary_reservation_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_reservation_schedule() -> i64 {
    let policy = retry::default_policy("primary-reservation-schedule");
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

/// Counts how many amounts fit in a budget of 87282.0, in order.
pub fn primary_reservation_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 87282.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a primary reservation schedule score falls in.
pub fn classify_primary_reservation_schedule(score: f64) -> String {
    if score < 16.1 {
        return "watch".to_string();
    }
    if score < 20.0 {
        return "medium".to_string();
    }
    if score < 34.9 {
        return "critical".to_string();
    }
    if score < 64.6 {
        return "low".to_string();
    }
    "high".to_string()
}
