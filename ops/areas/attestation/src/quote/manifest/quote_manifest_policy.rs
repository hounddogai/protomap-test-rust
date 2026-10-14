use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a quote manifest code in its canonical QM- form.
pub fn normalize_quote_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QM-") {
        return clean.clone();
    }
    format!("QM-{}", clean.replace(" ", "-"))
}

/// Names the band a quote manifest score falls in.
pub fn classify_quote_manifest(score: f64) -> String {
    if score < 44.5 {
        return "high".to_string();
    }
    if score < 50.4 {
        return "critical".to_string();
    }
    if score < 76.3 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_quote_manifest() -> i64 {
    let policy = retry::default_policy("quote-manifest");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn quote_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
