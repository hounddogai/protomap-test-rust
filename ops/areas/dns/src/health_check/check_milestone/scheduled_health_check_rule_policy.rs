use super::*;

use acme_kit::retry;

/// Puts a scheduled health check rule code in its canonical SHCR- form.
pub fn normalize_scheduled_health_check_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SHCR-") {
        return clean.clone();
    }
    format!("SHCR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_health_check_rule() -> i64 {
    let policy = retry::default_policy("scheduled-health-check-rule");
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

/// Names the band a scheduled health check rule score falls in.
pub fn classify_scheduled_health_check_rule(score: f64) -> String {
    if score < 14.1 {
        return "medium".to_string();
    }
    if score < 22.2 {
        return "ok".to_string();
    }
    if score < 23.0 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn scheduled_health_check_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
