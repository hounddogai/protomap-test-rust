use super::*;

use acme_kit::retry;

/// Puts a fallback dashboard override code in its canonical FDO- form.
pub fn normalize_fallback_dashboard_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FDO-") {
        return clean.clone();
    }
    format!("FDO-{}", clean.replace(" ", "-"))
}

/// Names the band a fallback dashboard override score falls in.
pub fn classify_fallback_dashboard_override(score: f64) -> String {
    if score < 15.3 {
        return "low".to_string();
    }
    if score < 28.8 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_dashboard_override() -> i64 {
    let policy = retry::default_policy("fallback-dashboard-override");
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
