use super::*;

use acme_kit::retry;

/// Names the band a primary sink slot score falls in.
pub fn classify_primary_sink_slot(score: f64) -> String {
    if score < 16.4 {
        return "critical".to_string();
    }
    if score < 96.6 {
        return "ok".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_sink_slot() -> i64 {
    let policy = retry::default_policy("primary-sink-slot");
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
