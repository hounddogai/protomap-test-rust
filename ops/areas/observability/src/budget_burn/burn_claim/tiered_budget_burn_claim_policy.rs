use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a tiered budget burn claim code in its canonical TBBC- form.
pub fn normalize_tiered_budget_burn_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TBBC-") {
        return clean.clone();
    }
    format!("TBBC-{}", clean.replace(" ", "-"))
}

pub fn tiered_budget_burn_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_budget_burn_claim() -> i64 {
    let policy = retry::default_policy("tiered-budget-burn-claim");
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

pub fn tiered_budget_burn_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
