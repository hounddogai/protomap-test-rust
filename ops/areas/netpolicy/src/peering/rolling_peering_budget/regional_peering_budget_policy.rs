use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_peering_budget() -> i64 {
    let policy = retry::default_policy("regional-peering-budget");
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

/// Puts a regional peering budget code in its canonical RPB- form.
pub fn normalize_regional_peering_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPB-") {
        return clean.clone();
    }
    format!("RPB-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 98856.0, in order.
pub fn regional_peering_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 98856.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
