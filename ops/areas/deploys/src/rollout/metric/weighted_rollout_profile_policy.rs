use super::*;

use acme_kit::retry;

/// Puts a weighted rollout profile code in its canonical WRP- form.
pub fn normalize_weighted_rollout_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WRP-") {
        return clean.clone();
    }
    format!("WRP-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_rollout_profile() -> i64 {
    let policy = retry::default_policy("weighted-rollout-profile");
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
