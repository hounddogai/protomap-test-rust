use super::*;

use acme_kit::retry;

/// Puts a regional runbook mapping code in its canonical RRM- form.
pub fn normalize_regional_runbook_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRM-") {
        return clean.clone();
    }
    format!("RRM-{}", clean.replace(" ", "-"))
}

/// Names the band a regional runbook mapping score falls in.
pub fn classify_regional_runbook_mapping(score: f64) -> String {
    if score < 47.8 {
        return "watch".to_string();
    }
    if score < 50.8 {
        return "high".to_string();
    }
    if score < 60.7 {
        return "medium".to_string();
    }
    if score < 100.0 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_runbook_mapping() -> i64 {
    let policy = retry::default_policy("regional-runbook-mapping");
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

pub fn regional_runbook_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
