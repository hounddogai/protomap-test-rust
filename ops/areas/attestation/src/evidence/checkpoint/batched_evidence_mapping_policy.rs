use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_evidence_mapping() -> i64 {
    let policy = retry::default_policy("batched-evidence-mapping");
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

pub fn batched_evidence_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a batched evidence mapping score falls in.
pub fn classify_batched_evidence_mapping(score: f64) -> String {
    if score < 7.0 {
        return "critical".to_string();
    }
    if score < 33.0 {
        return "low".to_string();
    }
    "ok".to_string()
}
