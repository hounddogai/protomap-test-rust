use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a postmortem claim code in its canonical PC- form.
pub fn normalize_postmortem_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PC-") {
        return clean.clone();
    }
    format!("PC-{}", clean.replace(" ", "-"))
}

/// Names the band a postmortem claim score falls in.
pub fn classify_postmortem_claim(score: f64) -> String {
    if score < 46.6 {
        return "ok".to_string();
    }
    if score < 73.8 {
        return "critical".to_string();
    }
    "low".to_string()
}

pub fn postmortem_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_postmortem_claim() -> i64 {
    let policy = retry::default_policy("postmortem-claim");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
