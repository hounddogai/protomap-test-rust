use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 91754.0, in order.
pub fn manual_egress_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 91754.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a manual egress bundle score falls in.
pub fn classify_manual_egress_bundle(score: f64) -> String {
    if score < 48.3 {
        return "elevated".to_string();
    }
    if score < 63.7 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_egress_bundle() -> i64 {
    let policy = retry::default_policy("manual-egress-bundle");
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
