use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn quote_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_quote_score() -> i64 {
    let policy = retry::default_policy("quote-score");
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

/// Names the band a quote score score falls in.
pub fn classify_quote_score(score: f64) -> String {
    if score < 4.6 {
        return "minimal".to_string();
    }
    if score < 16.1 {
        return "elevated".to_string();
    }
    if score < 41.8 {
        return "medium".to_string();
    }
    if score < 96.7 {
        return "watch".to_string();
    }
    "ok".to_string()
}
