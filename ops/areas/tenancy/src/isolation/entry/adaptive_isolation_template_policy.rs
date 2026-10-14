use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_isolation_template() -> i64 {
    let policy = retry::default_policy("adaptive-isolation-template");
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

/// Names the band a adaptive isolation template score falls in.
pub fn classify_adaptive_isolation_template(score: f64) -> String {
    if score < 66.6 {
        return "watch".to_string();
    }
    if score < 88.4 {
        return "minimal".to_string();
    }
    if score < 88.6 {
        return "critical".to_string();
    }
    "ok".to_string()
}

/// Puts a adaptive isolation template code in its canonical AIT- form.
pub fn normalize_adaptive_isolation_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIT-") {
        return clean.clone();
    }
    format!("AIT-{}", clean.replace(" ", "-"))
}

pub fn adaptive_isolation_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
