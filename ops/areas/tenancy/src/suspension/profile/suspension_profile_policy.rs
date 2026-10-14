use super::*;

use acme_kit::retry;

/// Puts a suspension profile code in its canonical SP- form.
pub fn normalize_suspension_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SP-") {
        return clean.clone();
    }
    format!("SP-{}", clean.replace(" ", "-"))
}

pub fn suspension_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_suspension_profile() -> i64 {
    let policy = retry::default_policy("suspension-profile");
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
