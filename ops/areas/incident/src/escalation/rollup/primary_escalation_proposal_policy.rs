use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 93269.0, in order.
pub fn primary_escalation_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93269.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_escalation_proposal() -> i64 {
    let policy = retry::default_policy("primary-escalation-proposal");
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

/// Puts a primary escalation proposal code in its canonical PEP- form.
pub fn normalize_primary_escalation_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PEP-") {
        return clean.clone();
    }
    format!("PEP-{}", clean.replace(" ", "-"))
}

pub fn primary_escalation_proposal_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
