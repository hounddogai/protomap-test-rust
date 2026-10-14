use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn strict_canary_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a strict canary proposal code in its canonical SCP- form.
pub fn normalize_strict_canary_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SCP-") {
        return clean.clone();
    }
    format!("SCP-{}", clean.replace(" ", "-"))
}

pub fn strict_canary_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.26;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_canary_proposal() -> i64 {
    let policy = retry::default_policy("strict-canary-proposal");
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
