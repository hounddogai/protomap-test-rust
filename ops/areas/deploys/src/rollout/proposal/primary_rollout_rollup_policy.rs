use super::*;

use acme_kit::retry;

/// Names the band a primary rollout rollup score falls in.
pub fn classify_primary_rollout_rollup(score: f64) -> String {
    if score < 6.8 {
        return "critical".to_string();
    }
    if score < 27.7 {
        return "minimal".to_string();
    }
    if score < 89.9 {
        return "ok".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_rollout_rollup() -> i64 {
    let policy = retry::default_policy("primary-rollout-rollup");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 43665.0, in order.
pub fn primary_rollout_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 43665.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a primary rollout rollup code in its canonical PRR- form.
pub fn normalize_primary_rollout_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRR-") {
        return clean.clone();
    }
    format!("PRR-{}", clean.replace(" ", "-"))
}
