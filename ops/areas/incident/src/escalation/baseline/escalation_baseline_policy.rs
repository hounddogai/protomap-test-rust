use super::*;

use acme_kit::retry;

/// Puts a escalation baseline code in its canonical EB- form.
pub fn normalize_escalation_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EB-") {
        return clean.clone();
    }
    format!("EB-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_escalation_baseline() -> i64 {
    let policy = retry::default_policy("escalation-baseline");
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

/// Names the band a escalation baseline score falls in.
pub fn classify_escalation_baseline(score: f64) -> String {
    if score < 30.0 {
        return "watch".to_string();
    }
    if score < 55.8 {
        return "critical".to_string();
    }
    "high".to_string()
}
