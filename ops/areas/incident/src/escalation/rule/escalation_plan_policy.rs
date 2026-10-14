use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 66520.0, in order.
pub fn escalation_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66520.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_escalation_plan() -> i64 {
    let policy = retry::default_policy("escalation-plan");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a escalation plan score falls in.
pub fn classify_escalation_plan(score: f64) -> String {
    if score < 32.7 {
        return "critical".to_string();
    }
    if score < 34.4 {
        return "ok".to_string();
    }
    if score < 60.9 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn merge_escalation_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
