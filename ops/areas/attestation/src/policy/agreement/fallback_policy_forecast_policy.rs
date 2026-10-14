use super::*;

use acme_kit::retry;

/// Puts a fallback policy forecast code in its canonical FPF- form.
pub fn normalize_fallback_policy_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FPF-") {
        return clean.clone();
    }
    format!("FPF-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_policy_forecast() -> i64 {
    let policy = retry::default_policy("fallback-policy-forecast");
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
