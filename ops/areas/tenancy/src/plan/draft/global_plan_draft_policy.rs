use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a global plan draft score falls in.
pub fn classify_global_plan_draft(score: f64) -> String {
    if score < 10.3 {
        return "ok".to_string();
    }
    if score < 31.4 {
        return "watch".to_string();
    }
    if score < 71.7 {
        return "elevated".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_plan_draft() -> i64 {
    let policy = retry::default_policy("global-plan-draft");
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

/// Puts a global plan draft code in its canonical GPD- form.
pub fn normalize_global_plan_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GPD-") {
        return clean.clone();
    }
    format!("GPD-{}", clean.replace(" ", "-"))
}

pub fn global_plan_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
