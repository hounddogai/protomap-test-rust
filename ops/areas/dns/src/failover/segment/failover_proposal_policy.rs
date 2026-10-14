use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a failover proposal score falls in.
pub fn classify_failover_proposal(score: f64) -> String {
    if score < 24.7 {
        return "medium".to_string();
    }
    if score < 39.9 {
        return "watch".to_string();
    }
    if score < 55.4 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_failover_proposal() -> i64 {
    let policy = retry::default_policy("failover-proposal");
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

pub fn failover_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.76;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn failover_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
