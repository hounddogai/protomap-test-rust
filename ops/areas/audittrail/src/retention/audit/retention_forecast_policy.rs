use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_retention_forecast() -> i64 {
    let policy = retry::default_policy("retention-forecast");
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

/// Puts a retention forecast code in its canonical RF- form.
pub fn normalize_retention_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RF-") {
        return clean.clone();
    }
    format!("RF-{}", clean.replace(" ", "-"))
}
