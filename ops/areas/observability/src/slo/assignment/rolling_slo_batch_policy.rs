use super::*;

use acme_kit::retry;

/// Puts a rolling slo batch code in its canonical RSB- form.
pub fn normalize_rolling_slo_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RSB-") {
        return clean.clone();
    }
    format!("RSB-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_slo_batch() -> i64 {
    let policy = retry::default_policy("rolling-slo-batch");
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
