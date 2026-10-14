use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 60696.0, in order.
pub fn overage_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 60696.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a overage preference score falls in.
pub fn classify_overage_preference(score: f64) -> String {
    if score < 9.4 {
        return "ok".to_string();
    }
    if score < 25.7 {
        return "watch".to_string();
    }
    if score < 87.7 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_overage_preference() -> i64 {
    let policy = retry::default_policy("overage-preference");
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

/// Puts a overage preference code in its canonical OP- form.
pub fn normalize_overage_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("OP-") {
        return clean.clone();
    }
    format!("OP-{}", clean.replace(" ", "-"))
}
