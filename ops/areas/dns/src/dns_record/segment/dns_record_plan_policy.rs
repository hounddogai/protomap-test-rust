use super::*;

use acme_kit::retry;

/// Names the band a dns record plan score falls in.
pub fn classify_dns_record_plan(score: f64) -> String {
    if score < 65.3 {
        return "minimal".to_string();
    }
    if score < 73.8 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dns_record_plan() -> i64 {
    let policy = retry::default_policy("dns-record-plan");
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
