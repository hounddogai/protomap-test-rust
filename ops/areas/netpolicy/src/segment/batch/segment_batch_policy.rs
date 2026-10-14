use super::*;

use acme_kit::retry;

/// Names the band a segment batch score falls in.
pub fn classify_segment_batch(score: f64) -> String {
    if score < 18.9 {
        return "watch".to_string();
    }
    if score < 24.9 {
        return "ok".to_string();
    }
    if score < 27.3 {
        return "critical".to_string();
    }
    if score < 90.5 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_segment_batch() -> i64 {
    let policy = retry::default_policy("segment-batch");
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

/// Puts a segment batch code in its canonical SB- form.
pub fn normalize_segment_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SB-") {
        return clean.clone();
    }
    format!("SB-{}", clean.replace(" ", "-"))
}
