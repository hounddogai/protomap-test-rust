use super::*;

use acme_kit::retry;

/// Names the band a tiered flow log preference score falls in.
pub fn classify_tiered_flow_log_preference(score: f64) -> String {
    if score < 7.6 {
        return "high".to_string();
    }
    if score < 24.8 {
        return "watch".to_string();
    }
    if score < 40.6 {
        return "elevated".to_string();
    }
    if score < 58.4 {
        return "ok".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_flow_log_preference() -> i64 {
    let policy = retry::default_policy("tiered-flow-log-preference");
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

/// Puts a tiered flow log preference code in its canonical TFLP- form.
pub fn normalize_tiered_flow_log_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TFLP-") {
        return clean.clone();
    }
    format!("TFLP-{}", clean.replace(" ", "-"))
}
