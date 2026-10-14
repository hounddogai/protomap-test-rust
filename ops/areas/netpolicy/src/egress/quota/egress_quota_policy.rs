use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_egress_quota() -> i64 {
    let policy = retry::default_policy("egress-quota");
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

pub fn egress_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 26398.0, in order.
pub fn egress_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 26398.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a egress quota score falls in.
pub fn classify_egress_quota(score: f64) -> String {
    if score < 53.6 {
        return "medium".to_string();
    }
    if score < 66.4 {
        return "watch".to_string();
    }
    if score < 83.3 {
        return "high".to_string();
    }
    if score < 91.3 {
        return "elevated".to_string();
    }
    "low".to_string()
}
