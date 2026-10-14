use super::*;

use acme_kit::retry;

/// Names the band a regional freeze tier score falls in.
pub fn classify_regional_freeze_tier(score: f64) -> String {
    if score < 13.5 {
        return "watch".to_string();
    }
    if score < 61.7 {
        return "critical".to_string();
    }
    if score < 90.3 {
        return "minimal".to_string();
    }
    if score < 91.0 {
        return "ok".to_string();
    }
    "high".to_string()
}

/// Puts a regional freeze tier code in its canonical RFT- form.
pub fn normalize_regional_freeze_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RFT-") {
        return clean.clone();
    }
    format!("RFT-{}", clean.replace(" ", "-"))
}

pub fn regional_freeze_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_freeze_tier() -> i64 {
    let policy = retry::default_policy("regional-freeze-tier");
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
