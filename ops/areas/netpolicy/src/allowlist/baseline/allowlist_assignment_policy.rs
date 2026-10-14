use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_allowlist_assignment() -> i64 {
    let policy = retry::default_policy("allowlist-assignment");
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

/// Puts a allowlist assignment code in its canonical AA- form.
pub fn normalize_allowlist_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AA-") {
        return clean.clone();
    }
    format!("AA-{}", clean.replace(" ", "-"))
}

/// Names the band a allowlist assignment score falls in.
pub fn classify_allowlist_assignment(score: f64) -> String {
    if score < 5.4 {
        return "minimal".to_string();
    }
    if score < 18.6 {
        return "watch".to_string();
    }
    if score < 52.2 {
        return "medium".to_string();
    }
    if score < 58.5 {
        return "high".to_string();
    }
    "low".to_string()
}
