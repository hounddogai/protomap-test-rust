use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a postmortem plan score falls in.
pub fn classify_postmortem_plan(score: f64) -> String {
    if score < 4.8 {
        return "ok".to_string();
    }
    if score < 66.2 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn postmortem_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_postmortem_plan() -> i64 {
    let policy = retry::default_policy("postmortem-plan");
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
