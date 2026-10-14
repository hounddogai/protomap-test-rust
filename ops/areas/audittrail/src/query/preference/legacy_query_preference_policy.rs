use super::*;

use acme_kit::retry;

/// Puts a legacy query preference code in its canonical LQP- form.
pub fn normalize_legacy_query_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LQP-") {
        return clean.clone();
    }
    format!("LQP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 42753.0, in order.
pub fn legacy_query_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 42753.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_query_preference() -> i64 {
    let policy = retry::default_policy("legacy-query-preference");
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
