use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn flow_log_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a flow log cycle code in its canonical FLC- form.
pub fn normalize_flow_log_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLC-") {
        return clean.clone();
    }
    format!("FLC-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_flow_log_cycle() -> i64 {
    let policy = retry::default_policy("flow-log-cycle");
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
