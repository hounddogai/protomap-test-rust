use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_quote_plan() -> i64 {
    let policy = retry::default_policy("manual-quote-plan");
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

pub fn manual_quote_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a manual quote plan score falls in.
pub fn classify_manual_quote_plan(score: f64) -> String {
    if score < 21.0 {
        return "watch".to_string();
    }
    if score < 51.6 {
        return "ok".to_string();
    }
    "medium".to_string()
}
