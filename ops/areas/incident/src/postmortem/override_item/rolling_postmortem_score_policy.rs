use super::*;

use acme_kit::retry;

/// Puts a rolling postmortem score code in its canonical RPS- form.
pub fn normalize_rolling_postmortem_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPS-") {
        return clean.clone();
    }
    format!("RPS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_postmortem_score() -> i64 {
    let policy = retry::default_policy("rolling-postmortem-score");
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
