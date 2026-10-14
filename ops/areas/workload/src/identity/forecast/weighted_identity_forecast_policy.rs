use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_identity_forecast() -> i64 {
    let policy = retry::default_policy("weighted-identity-forecast");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a weighted identity forecast code in its canonical WIF- form.
pub fn normalize_weighted_identity_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WIF-") {
        return clean.clone();
    }
    format!("WIF-{}", clean.replace(" ", "-"))
}
