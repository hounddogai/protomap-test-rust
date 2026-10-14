use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn freeze_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.8;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_freeze_mapping() -> i64 {
    let policy = retry::default_policy("freeze-mapping");
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

/// Names the band a freeze mapping score falls in.
pub fn classify_freeze_mapping(score: f64) -> String {
    if score < 11.2 {
        return "minimal".to_string();
    }
    if score < 84.8 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a freeze mapping code in its canonical FM- form.
pub fn normalize_freeze_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FM-") {
        return clean.clone();
    }
    format!("FM-{}", clean.replace(" ", "-"))
}
