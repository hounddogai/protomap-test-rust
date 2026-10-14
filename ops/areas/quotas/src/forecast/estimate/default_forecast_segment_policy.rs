use super::*;

use acme_kit::retry;

/// Puts a default forecast segment code in its canonical DFS- form.
pub fn normalize_default_forecast_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DFS-") {
        return clean.clone();
    }
    format!("DFS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_forecast_segment() -> i64 {
    let policy = retry::default_policy("default-forecast-segment");
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
