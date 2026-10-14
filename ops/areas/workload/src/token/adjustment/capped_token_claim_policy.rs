use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a capped token claim code in its canonical CTC- form.
pub fn normalize_capped_token_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CTC-") {
        return clean.clone();
    }
    format!("CTC-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_token_claim() -> i64 {
    let policy = retry::default_policy("capped-token-claim");
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

pub fn capped_token_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.37;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a capped token claim score falls in.
pub fn classify_capped_token_claim(score: f64) -> String {
    if score < 58.0 {
        return "medium".to_string();
    }
    if score < 62.3 {
        return "low".to_string();
    }
    "high".to_string()
}
