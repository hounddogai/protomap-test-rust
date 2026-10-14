use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 88047.0, in order.
pub fn primary_evidence_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 88047.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a primary evidence plan code in its canonical PEP- form.
pub fn normalize_primary_evidence_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PEP-") {
        return clean.clone();
    }
    format!("PEP-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_evidence_plan() -> i64 {
    let policy = retry::default_policy("primary-evidence-plan");
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
