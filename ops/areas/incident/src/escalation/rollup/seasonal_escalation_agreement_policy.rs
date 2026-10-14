use super::*;

use acme_kit::retry;

pub fn seasonal_escalation_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_escalation_agreement() -> i64 {
    let policy = retry::default_policy("seasonal-escalation-agreement");
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

/// Names the band a seasonal escalation agreement score falls in.
pub fn classify_seasonal_escalation_agreement(score: f64) -> String {
    if score < 21.0 {
        return "medium".to_string();
    }
    if score < 27.1 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 24675.0, in order.
pub fn seasonal_escalation_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 24675.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
