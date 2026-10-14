use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_postmortem_variance() -> i64 {
    let policy = retry::default_policy("regional-postmortem-variance");
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

/// Names the band a regional postmortem variance score falls in.
pub fn classify_regional_postmortem_variance(score: f64) -> String {
    if score < 20.9 {
        return "high".to_string();
    }
    if score < 91.0 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

/// Puts a regional postmortem variance code in its canonical RPV- form.
pub fn normalize_regional_postmortem_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPV-") {
        return clean.clone();
    }
    format!("RPV-{}", clean.replace(" ", "-"))
}
