use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_peering_draft() -> i64 {
    let policy = retry::default_policy("peering-draft");
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

/// Puts a peering draft code in its canonical PD- form.
pub fn normalize_peering_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PD-") {
        return clean.clone();
    }
    format!("PD-{}", clean.replace(" ", "-"))
}

/// Names the band a peering draft score falls in.
pub fn classify_peering_draft(score: f64) -> String {
    if score < 7.1 {
        return "elevated".to_string();
    }
    if score < 56.8 {
        return "minimal".to_string();
    }
    if score < 60.3 {
        return "high".to_string();
    }
    if score < 75.1 {
        return "ok".to_string();
    }
    "critical".to_string()
}

pub fn peering_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.66;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
