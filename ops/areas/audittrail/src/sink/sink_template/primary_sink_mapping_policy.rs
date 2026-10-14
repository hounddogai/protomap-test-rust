use super::*;

use acme_kit::retry;

/// Names the band a primary sink mapping score falls in.
pub fn classify_primary_sink_mapping(score: f64) -> String {
    if score < 60.6 {
        return "minimal".to_string();
    }
    if score < 91.8 {
        return "high".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_sink_mapping() -> i64 {
    let policy = retry::default_policy("primary-sink-mapping");
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
