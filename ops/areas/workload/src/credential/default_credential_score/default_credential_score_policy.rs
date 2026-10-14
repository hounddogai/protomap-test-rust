use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_credential_score() -> i64 {
    let policy = retry::default_policy("default-credential-score");
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

/// Names the band a default credential score score falls in.
pub fn classify_default_credential_score(score: f64) -> String {
    if score < 67.2 {
        return "medium".to_string();
    }
    if score < 91.2 {
        return "minimal".to_string();
    }
    if score < 92.6 {
        return "ok".to_string();
    }
    "high".to_string()
}

pub fn default_credential_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.54;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
