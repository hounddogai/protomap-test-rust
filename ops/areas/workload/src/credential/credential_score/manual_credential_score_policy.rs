use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn manual_credential_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Names the band a manual credential score score falls in.
pub fn classify_manual_credential_score(score: f64) -> String {
    if score < 12.1 {
        return "watch".to_string();
    }
    if score < 24.2 {
        return "low".to_string();
    }
    if score < 57.6 {
        return "critical".to_string();
    }
    if score < 86.7 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn manual_credential_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_credential_score() -> i64 {
    let policy = retry::default_policy("manual-credential-score");
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
