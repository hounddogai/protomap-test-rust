use super::*;

use acme_kit::retry;

/// Names the band a primary limit quota score falls in.
pub fn classify_primary_limit_quota(score: f64) -> String {
    if score < 13.3 {
        return "watch".to_string();
    }
    if score < 46.2 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 74205.0, in order.
pub fn primary_limit_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74205.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_limit_quota() -> i64 {
    let policy = retry::default_policy("primary-limit-quota");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a primary limit quota code in its canonical PLQ- form.
pub fn normalize_primary_limit_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PLQ-") {
        return clean.clone();
    }
    format!("PLQ-{}", clean.replace(" ", "-"))
}
