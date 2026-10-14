use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_trust_domain_adjustment() -> i64 {
    let policy = retry::default_policy("trust-domain-adjustment");
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

/// Puts a trust domain adjustment code in its canonical TDA- form.
pub fn normalize_trust_domain_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDA-") {
        return clean.clone();
    }
    format!("TDA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 24249.0, in order.
pub fn trust_domain_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 24249.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn trust_domain_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.62;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
