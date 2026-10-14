use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a tiered rollout proposal code in its canonical TRP- form.
pub fn normalize_tiered_rollout_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TRP-") {
        return clean.clone();
    }
    format!("TRP-{}", clean.replace(" ", "-"))
}

pub fn tiered_rollout_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_rollout_proposal() -> i64 {
    let policy = retry::default_policy("tiered-rollout-proposal");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
