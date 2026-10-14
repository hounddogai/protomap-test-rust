use super::*;

use acme_kit::retry;

/// Puts a policy agreement code in its canonical PA- form.
pub fn normalize_policy_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PA-") {
        return clean.clone();
    }
    format!("PA-{}", clean.replace(" ", "-"))
}

/// Names the band a policy agreement score falls in.
pub fn classify_policy_agreement(score: f64) -> String {
    if score < 86.6 {
        return "elevated".to_string();
    }
    if score < 92.7 {
        return "high".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_policy_agreement() -> i64 {
    let policy = retry::default_policy("policy-agreement");
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
