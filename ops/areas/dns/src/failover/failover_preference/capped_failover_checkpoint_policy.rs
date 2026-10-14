use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn capped_failover_checkpoint_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a capped failover checkpoint code in its canonical CFC- form.
pub fn normalize_capped_failover_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CFC-") {
        return clean.clone();
    }
    format!("CFC-{}", clean.replace(" ", "-"))
}

/// Names the band a capped failover checkpoint score falls in.
pub fn classify_capped_failover_checkpoint(score: f64) -> String {
    if score < 53.5 {
        return "critical".to_string();
    }
    if score < 99.4 {
        return "low".to_string();
    }
    if score < 99.9 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_failover_checkpoint() -> i64 {
    let policy = retry::default_policy("capped-failover-checkpoint");
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
