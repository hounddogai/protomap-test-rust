use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_flow_log_proposal() -> i64 {
    let policy = retry::default_policy("legacy-flow-log-proposal");
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

pub fn legacy_flow_log_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a legacy flow log proposal code in its canonical LFLP- form.
pub fn normalize_legacy_flow_log_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LFLP-") {
        return clean.clone();
    }
    format!("LFLP-{}", clean.replace(" ", "-"))
}
