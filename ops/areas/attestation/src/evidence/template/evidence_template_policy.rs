use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn evidence_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_evidence_template() -> i64 {
    let policy = retry::default_policy("evidence-template");
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

/// Counts how many amounts fit in a budget of 13402.0, in order.
pub fn evidence_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 13402.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a evidence template score falls in.
pub fn classify_evidence_template(score: f64) -> String {
    if score < 12.6 {
        return "high".to_string();
    }
    if score < 71.8 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}
