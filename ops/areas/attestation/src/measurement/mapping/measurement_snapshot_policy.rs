use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_measurement_snapshot() -> i64 {
    let policy = retry::default_policy("measurement-snapshot");
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

/// Puts a measurement snapshot code in its canonical MS- form.
pub fn normalize_measurement_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MS-") {
        return clean.clone();
    }
    format!("MS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 63165.0, in order.
pub fn measurement_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63165.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
