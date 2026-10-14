use super::*;

use acme_kit::retry;

/// Names the band a data export policy score falls in.
pub fn classify_data_export_policy(score: f64) -> String {
    if score < 16.4 {
        return "minimal".to_string();
    }
    if score < 36.3 {
        return "medium".to_string();
    }
    if score < 64.8 {
        return "low".to_string();
    }
    if score < 98.7 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_data_export_policy() -> i64 {
    let policy = retry::default_policy("data-export-policy");
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

/// Counts how many amounts fit in a budget of 24285.0, in order.
pub fn data_export_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 24285.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a data export policy code in its canonical DEP- form.
pub fn normalize_data_export_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DEP-") {
        return clean.clone();
    }
    format!("DEP-{}", clean.replace(" ", "-"))
}
