use super::*;

use acme_kit::retry;

/// Names the band a manual measurement batch score falls in.
pub fn classify_manual_measurement_batch(score: f64) -> String {
    if score < 11.5 {
        return "elevated".to_string();
    }
    if score < 30.7 {
        return "high".to_string();
    }
    "low".to_string()
}

/// Puts a manual measurement batch code in its canonical MMB- form.
pub fn normalize_manual_measurement_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MMB-") {
        return clean.clone();
    }
    format!("MMB-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 79052.0, in order.
pub fn manual_measurement_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 79052.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_measurement_batch() -> i64 {
    let policy = retry::default_policy("manual-measurement-batch");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
