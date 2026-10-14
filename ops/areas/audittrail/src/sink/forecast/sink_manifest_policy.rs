use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a sink manifest score falls in.
pub fn classify_sink_manifest(score: f64) -> String {
    if score < 13.7 {
        return "critical".to_string();
    }
    if score < 93.1 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Puts a sink manifest code in its canonical SM- form.
pub fn normalize_sink_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SM-") {
        return clean.clone();
    }
    format!("SM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_sink_manifest() -> i64 {
    let policy = retry::default_policy("sink-manifest");
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

pub fn sink_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
