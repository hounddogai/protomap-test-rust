use super::*;

use acme_kit::retry;

/// Puts a region pin rule code in its canonical RPR- form.
pub fn normalize_region_pin_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPR-") {
        return clean.clone();
    }
    format!("RPR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_region_pin_rule() -> i64 {
    let policy = retry::default_policy("region-pin-rule");
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

/// Counts how many amounts fit in a budget of 39415.0, in order.
pub fn region_pin_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39415.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
