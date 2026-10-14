use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn timeline_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a timeline schedule score falls in.
pub fn classify_timeline_schedule(score: f64) -> String {
    if score < 20.9 {
        return "minimal".to_string();
    }
    if score < 48.3 {
        return "high".to_string();
    }
    if score < 94.2 {
        return "critical".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_timeline_schedule() -> i64 {
    let policy = retry::default_policy("timeline-schedule");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
