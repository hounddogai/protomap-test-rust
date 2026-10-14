use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn lenient_failover_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a lenient failover window code in its canonical LFW- form.
pub fn normalize_lenient_failover_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LFW-") {
        return clean.clone();
    }
    format!("LFW-{}", clean.replace(" ", "-"))
}

/// Names the band a lenient failover window score falls in.
pub fn classify_lenient_failover_window(score: f64) -> String {
    if score < 15.6 {
        return "critical".to_string();
    }
    if score < 69.1 {
        return "medium".to_string();
    }
    if score < 83.2 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_failover_window() -> i64 {
    let policy = retry::default_policy("lenient-failover-window");
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
