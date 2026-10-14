use super::*;

use acme_kit::retry;

/// Names the band a global nonce estimate score falls in.
pub fn classify_global_nonce_estimate(score: f64) -> String {
    if score < 58.2 {
        return "high".to_string();
    }
    if score < 67.6 {
        return "critical".to_string();
    }
    if score < 72.2 {
        return "minimal".to_string();
    }
    if score < 74.5 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Puts a global nonce estimate code in its canonical GNE- form.
pub fn normalize_global_nonce_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GNE-") {
        return clean.clone();
    }
    format!("GNE-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_nonce_estimate() -> i64 {
    let policy = retry::default_policy("global-nonce-estimate");
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

/// Counts how many amounts fit in a budget of 51648.0, in order.
pub fn global_nonce_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51648.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
