use super::*;

use acme_kit::retry;

/// Puts a tiered usage summary code in its canonical TUS- form.
pub fn normalize_tiered_usage_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TUS-") {
        return clean.clone();
    }
    format!("TUS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 99714.0, in order.
pub fn tiered_usage_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 99714.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_usage_summary() -> i64 {
    let policy = retry::default_policy("tiered-usage-summary");
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
