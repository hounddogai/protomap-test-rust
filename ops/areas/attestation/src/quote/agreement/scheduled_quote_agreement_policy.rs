use super::*;

use acme_kit::retry;

pub fn scheduled_quote_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a scheduled quote agreement code in its canonical SQA- form.
pub fn normalize_scheduled_quote_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SQA-") {
        return clean.clone();
    }
    format!("SQA-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_quote_agreement() -> i64 {
    let policy = retry::default_policy("scheduled-quote-agreement");
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

/// Counts how many amounts fit in a budget of 75747.0, in order.
pub fn scheduled_quote_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 75747.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
