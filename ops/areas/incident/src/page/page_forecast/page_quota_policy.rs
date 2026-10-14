use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_page_quota() -> i64 {
    let policy = retry::default_policy("page-quota");
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

/// Names the band a page quota score falls in.
pub fn classify_page_quota(score: f64) -> String {
    if score < 63.9 {
        return "watch".to_string();
    }
    if score < 75.3 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn page_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a page quota code in its canonical PQ- form.
pub fn normalize_page_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PQ-") {
        return clean.clone();
    }
    format!("PQ-{}", clean.replace(" ", "-"))
}
