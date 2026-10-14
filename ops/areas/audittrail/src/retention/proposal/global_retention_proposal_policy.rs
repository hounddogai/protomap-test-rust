use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn global_retention_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_retention_proposal() -> i64 {
    let policy = retry::default_policy("global-retention-proposal");
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

/// Puts a global retention proposal code in its canonical GRP- form.
pub fn normalize_global_retention_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GRP-") {
        return clean.clone();
    }
    format!("GRP-{}", clean.replace(" ", "-"))
}
