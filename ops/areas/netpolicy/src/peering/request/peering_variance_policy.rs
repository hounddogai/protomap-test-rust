use super::*;

use acme_kit::retry;

/// Puts a peering variance code in its canonical PV- form.
pub fn normalize_peering_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PV-") {
        return clean.clone();
    }
    format!("PV-{}", clean.replace(" ", "-"))
}

/// Names the band a peering variance score falls in.
pub fn classify_peering_variance(score: f64) -> String {
    if score < 4.7 {
        return "ok".to_string();
    }
    if score < 62.3 {
        return "minimal".to_string();
    }
    if score < 73.9 {
        return "low".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_peering_variance() -> i64 {
    let policy = retry::default_policy("peering-variance");
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
