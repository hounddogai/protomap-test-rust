use super::*;

use acme_kit::retry;

/// Names the band a page rule score falls in.
pub fn classify_page_rule(score: f64) -> String {
    if score < 38.2 {
        return "ok".to_string();
    }
    if score < 53.3 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_page_rule() -> i64 {
    let policy = retry::default_policy("page-rule");
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
