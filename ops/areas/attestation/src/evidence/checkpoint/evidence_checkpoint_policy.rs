use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_evidence_checkpoint() -> i64 {
    let policy = retry::default_policy("evidence-checkpoint");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 47847.0, in order.
pub fn evidence_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47847.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a evidence checkpoint code in its canonical EC- form.
pub fn normalize_evidence_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EC-") {
        return clean.clone();
    }
    format!("EC-{}", clean.replace(" ", "-"))
}
