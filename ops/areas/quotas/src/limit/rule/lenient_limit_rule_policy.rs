use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_limit_rule() -> i64 {
    let policy = retry::default_policy("lenient-limit-rule");
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

pub fn lenient_limit_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Puts a lenient limit rule code in its canonical LLR- form.
pub fn normalize_lenient_limit_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LLR-") {
        return clean.clone();
    }
    format!("LLR-{}", clean.replace(" ", "-"))
}
