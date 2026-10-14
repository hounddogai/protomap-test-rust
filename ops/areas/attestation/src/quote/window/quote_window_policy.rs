use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_quote_window() -> i64 {
    let policy = retry::default_policy("quote-window");
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

pub fn quote_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.59;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a quote window score falls in.
pub fn classify_quote_window(score: f64) -> String {
    if score < 3.1 {
        return "minimal".to_string();
    }
    if score < 13.1 {
        return "elevated".to_string();
    }
    if score < 64.5 {
        return "low".to_string();
    }
    "watch".to_string()
}

/// Puts a quote window code in its canonical QW- form.
pub fn normalize_quote_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QW-") {
        return clean.clone();
    }
    format!("QW-{}", clean.replace(" ", "-"))
}
