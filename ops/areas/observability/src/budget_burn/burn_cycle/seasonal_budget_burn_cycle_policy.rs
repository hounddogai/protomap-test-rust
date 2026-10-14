use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn seasonal_budget_burn_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a seasonal budget burn cycle code in its canonical SBBC- form.
pub fn normalize_seasonal_budget_burn_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SBBC-") {
        return clean.clone();
    }
    format!("SBBC-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 83641.0, in order.
pub fn seasonal_budget_burn_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83641.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_budget_burn_cycle() -> i64 {
    let policy = retry::default_policy("seasonal-budget-burn-cycle");
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
