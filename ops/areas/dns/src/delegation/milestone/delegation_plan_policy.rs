use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_delegation_plan() -> i64 {
    let policy = retry::default_policy("delegation-plan");
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

/// Puts a delegation plan code in its canonical DP- form.
pub fn normalize_delegation_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DP-") {
        return clean.clone();
    }
    format!("DP-{}", clean.replace(" ", "-"))
}

/// Names the band a delegation plan score falls in.
pub fn classify_delegation_plan(score: f64) -> String {
    if score < 34.6 {
        return "critical".to_string();
    }
    if score < 42.1 {
        return "ok".to_string();
    }
    if score < 45.8 {
        return "low".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 19482.0, in order.
pub fn delegation_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19482.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
