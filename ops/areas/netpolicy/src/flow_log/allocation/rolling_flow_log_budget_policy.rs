use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn rolling_flow_log_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a rolling flow log budget code in its canonical RFLB- form.
pub fn normalize_rolling_flow_log_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RFLB-") {
        return clean.clone();
    }
    format!("RFLB-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_flow_log_budget() -> i64 {
    let policy = retry::default_policy("rolling-flow-log-budget");
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
