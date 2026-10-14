use super::*;

use acme_kit::retry;

pub fn capped_query_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Puts a capped query mapping code in its canonical CQM- form.
pub fn normalize_capped_query_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CQM-") {
        return clean.clone();
    }
    format!("CQM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_query_mapping() -> i64 {
    let policy = retry::default_policy("capped-query-mapping");
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

/// Counts how many amounts fit in a budget of 79362.0, in order.
pub fn capped_query_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 79362.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
