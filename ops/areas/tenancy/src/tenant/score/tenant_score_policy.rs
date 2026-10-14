use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn tenant_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a tenant score code in its canonical TS- form.
pub fn normalize_tenant_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TS-") {
        return clean.clone();
    }
    format!("TS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tenant_score() -> i64 {
    let policy = retry::default_policy("tenant-score");
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

/// Names the band a tenant score score falls in.
pub fn classify_tenant_score(score: f64) -> String {
    if score < 4.3 {
        return "low".to_string();
    }
    if score < 65.8 {
        return "ok".to_string();
    }
    if score < 79.7 {
        return "watch".to_string();
    }
    if score < 84.6 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
