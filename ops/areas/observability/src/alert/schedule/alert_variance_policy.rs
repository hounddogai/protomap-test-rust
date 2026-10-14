use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_alert_variance() -> i64 {
    let policy = retry::default_policy("alert-variance");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a alert variance code in its canonical AV- form.
pub fn normalize_alert_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AV-") {
        return clean.clone();
    }
    format!("AV-{}", clean.replace(" ", "-"))
}

/// Names the band a alert variance score falls in.
pub fn classify_alert_variance(score: f64) -> String {
    if score < 12.3 {
        return "low".to_string();
    }
    if score < 40.8 {
        return "minimal".to_string();
    }
    if score < 77.1 {
        return "medium".to_string();
    }
    if score < 90.9 {
        return "watch".to_string();
    }
    "critical".to_string()
}
