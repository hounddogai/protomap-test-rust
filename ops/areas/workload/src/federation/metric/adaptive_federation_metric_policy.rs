use super::*;

use acme_kit::retry;

/// Puts a adaptive federation metric code in its canonical AFM- form.
pub fn normalize_adaptive_federation_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AFM-") {
        return clean.clone();
    }
    format!("AFM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_federation_metric() -> i64 {
    let policy = retry::default_policy("adaptive-federation-metric");
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
