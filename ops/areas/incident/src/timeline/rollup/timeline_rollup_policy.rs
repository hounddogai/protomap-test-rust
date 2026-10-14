use super::*;

use acme_kit::retry;

/// Names the band a timeline rollup score falls in.
pub fn classify_timeline_rollup(score: f64) -> String {
    if score < 1.1 {
        return "low".to_string();
    }
    if score < 84.9 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a timeline rollup code in its canonical TR- form.
pub fn normalize_timeline_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TR-") {
        return clean.clone();
    }
    format!("TR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_timeline_rollup() -> i64 {
    let policy = retry::default_policy("timeline-rollup");
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
