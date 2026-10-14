use super::*;

use acme_kit::retry;

/// Puts a approval forecast code in its canonical AF- form.
pub fn normalize_approval_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AF-") {
        return clean.clone();
    }
    format!("AF-{}", clean.replace(" ", "-"))
}

pub fn approval_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_approval_forecast() -> i64 {
    let policy = retry::default_policy("approval-forecast");
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
