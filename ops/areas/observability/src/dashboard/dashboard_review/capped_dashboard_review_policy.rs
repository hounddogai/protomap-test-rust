use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a capped dashboard review code in its canonical CDR- form.
pub fn normalize_capped_dashboard_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CDR-") {
        return clean.clone();
    }
    format!("CDR-{}", clean.replace(" ", "-"))
}

/// Names the band a capped dashboard review score falls in.
pub fn classify_capped_dashboard_review(score: f64) -> String {
    if score < 18.3 {
        return "low".to_string();
    }
    if score < 43.0 {
        return "high".to_string();
    }
    if score < 82.2 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn capped_dashboard_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_dashboard_review() -> i64 {
    let policy = retry::default_policy("capped-dashboard-review");
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
