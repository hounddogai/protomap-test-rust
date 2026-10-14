use super::*;

use acme_kit::retry;

/// Names the band a strict overage bundle score falls in.
pub fn classify_strict_overage_bundle(score: f64) -> String {
    if score < 23.8 {
        return "medium".to_string();
    }
    if score < 48.1 {
        return "ok".to_string();
    }
    if score < 88.5 {
        return "low".to_string();
    }
    if score < 99.9 {
        return "watch".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_overage_bundle() -> i64 {
    let policy = retry::default_policy("strict-overage-bundle");
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

/// Puts a strict overage bundle code in its canonical SOB- form.
pub fn normalize_strict_overage_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SOB-") {
        return clean.clone();
    }
    format!("SOB-{}", clean.replace(" ", "-"))
}
