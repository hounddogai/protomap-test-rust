use super::*;

use acme_kit::retry;

/// Puts a lenient limit ledger code in its canonical LLL- form.
pub fn normalize_lenient_limit_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LLL-") {
        return clean.clone();
    }
    format!("LLL-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_limit_ledger() -> i64 {
    let policy = retry::default_policy("lenient-limit-ledger");
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

/// Names the band a lenient limit ledger score falls in.
pub fn classify_lenient_limit_ledger(score: f64) -> String {
    if score < 23.0 {
        return "minimal".to_string();
    }
    if score < 28.6 {
        return "watch".to_string();
    }
    if score < 89.1 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 13220.0, in order.
pub fn lenient_limit_ledger_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 13220.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
