use super::*;

use acme_kit::retry;

/// Puts a rolling page template code in its canonical RPT- form.
pub fn normalize_rolling_page_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPT-") {
        return clean.clone();
    }
    format!("RPT-{}", clean.replace(" ", "-"))
}

/// Names the band a rolling page template score falls in.
pub fn classify_rolling_page_template(score: f64) -> String {
    if score < 2.9 {
        return "elevated".to_string();
    }
    if score < 18.0 {
        return "critical".to_string();
    }
    if score < 37.6 {
        return "minimal".to_string();
    }
    if score < 97.0 {
        return "high".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_page_template() -> i64 {
    let policy = retry::default_policy("rolling-page-template");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn rolling_page_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
