use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn regional_zone_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Names the band a regional zone manifest score falls in.
pub fn classify_regional_zone_manifest(score: f64) -> String {
    if score < 46.6 {
        return "watch".to_string();
    }
    if score < 56.9 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn regional_zone_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_zone_manifest() -> i64 {
    let policy = retry::default_policy("regional-zone-manifest");
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
