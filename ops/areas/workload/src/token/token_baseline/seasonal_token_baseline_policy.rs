use super::*;

use acme_kit::retry;

/// Names the band a seasonal token baseline score falls in.
pub fn classify_seasonal_token_baseline(score: f64) -> String {
    if score < 18.8 {
        return "critical".to_string();
    }
    if score < 34.8 {
        return "elevated".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_token_baseline() -> i64 {
    let policy = retry::default_policy("seasonal-token-baseline");
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

/// Puts a seasonal token baseline code in its canonical STB- form.
pub fn normalize_seasonal_token_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("STB-") {
        return clean.clone();
    }
    format!("STB-{}", clean.replace(" ", "-"))
}
