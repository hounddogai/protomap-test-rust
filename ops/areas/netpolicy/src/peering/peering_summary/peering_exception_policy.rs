use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 66420.0, in order.
pub fn peering_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66420.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_peering_exception() -> i64 {
    let policy = retry::default_policy("peering-exception");
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

/// Names the band a peering exception score falls in.
pub fn classify_peering_exception(score: f64) -> String {
    if score < 12.7 {
        return "ok".to_string();
    }
    if score < 31.8 {
        return "high".to_string();
    }
    if score < 66.9 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

/// Puts a peering exception code in its canonical PE- form.
pub fn normalize_peering_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PE-") {
        return clean.clone();
    }
    format!("PE-{}", clean.replace(" ", "-"))
}
