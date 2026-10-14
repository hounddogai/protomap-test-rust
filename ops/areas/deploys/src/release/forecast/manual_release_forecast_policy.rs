use super::*;

use acme_kit::retry;

/// Puts a manual release forecast code in its canonical MRF- form.
pub fn normalize_manual_release_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MRF-") {
        return clean.clone();
    }
    format!("MRF-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 54763.0, in order.
pub fn manual_release_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 54763.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_release_forecast() -> i64 {
    let policy = retry::default_policy("manual-release-forecast");
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
