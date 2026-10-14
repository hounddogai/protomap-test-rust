use super::*;

use acme_kit::retry;

/// Puts a freeze policy code in its canonical FP- form.
pub fn normalize_freeze_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FP-") {
        return clean.clone();
    }
    format!("FP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 2388.0, in order.
pub fn freeze_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2388.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_freeze_policy() -> i64 {
    let policy = retry::default_policy("freeze-policy");
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
