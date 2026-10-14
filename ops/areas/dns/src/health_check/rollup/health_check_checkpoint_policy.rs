use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 24065.0, in order.
pub fn health_check_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 24065.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_health_check_checkpoint() -> i64 {
    let policy = retry::default_policy("health-check-checkpoint");
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

/// Names the band a health check checkpoint score falls in.
pub fn classify_health_check_checkpoint(score: f64) -> String {
    if score < 15.0 {
        return "low".to_string();
    }
    if score < 39.0 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a health check checkpoint code in its canonical HCC- form.
pub fn normalize_health_check_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCC-") {
        return clean.clone();
    }
    format!("HCC-{}", clean.replace(" ", "-"))
}
