use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a weighted page score score falls in.
pub fn classify_weighted_page_score(score: f64) -> String {
    if score < 43.4 {
        return "medium".to_string();
    }
    if score < 43.5 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn weighted_page_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_page_score() -> i64 {
    let policy = retry::default_policy("weighted-page-score");
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

pub fn weighted_page_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
