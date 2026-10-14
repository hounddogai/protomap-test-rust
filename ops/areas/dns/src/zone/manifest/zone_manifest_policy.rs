use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_zone_manifest() -> i64 {
    let policy = retry::default_policy("zone-manifest");
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

pub fn zone_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a zone manifest score falls in.
pub fn classify_zone_manifest(score: f64) -> String {
    if score < 54.8 {
        return "minimal".to_string();
    }
    if score < 81.4 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn zone_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.04;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
