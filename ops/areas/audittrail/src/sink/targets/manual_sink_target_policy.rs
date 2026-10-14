use super::*;

use acme_kit::retry;

/// Names the band a manual sink target score falls in.
pub fn classify_manual_sink_target(score: f64) -> String {
    if score < 34.2 {
        return "medium".to_string();
    }
    if score < 79.3 {
        return "high".to_string();
    }
    "critical".to_string()
}

pub fn manual_sink_target_span(start: i64, end: i64) -> i64 {
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
pub fn probe_manual_sink_target() -> i64 {
    let policy = retry::default_policy("manual-sink-target");
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

/// Puts a manual sink target code in its canonical MST- form.
pub fn normalize_manual_sink_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MST-") {
        return clean.clone();
    }
    format!("MST-{}", clean.replace(" ", "-"))
}
