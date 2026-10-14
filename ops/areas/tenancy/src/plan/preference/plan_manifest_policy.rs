use super::*;

use acme_kit::retry;

pub fn plan_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_plan_manifest() -> i64 {
    let policy = retry::default_policy("plan-manifest");
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

/// Names the band a plan manifest score falls in.
pub fn classify_plan_manifest(score: f64) -> String {
    if score < 2.8 {
        return "high".to_string();
    }
    if score < 11.6 {
        return "elevated".to_string();
    }
    if score < 73.7 {
        return "low".to_string();
    }
    if score < 82.9 {
        return "ok".to_string();
    }
    "critical".to_string()
}

/// Puts a plan manifest code in its canonical PM- form.
pub fn normalize_plan_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PM-") {
        return clean.clone();
    }
    format!("PM-{}", clean.replace(" ", "-"))
}
