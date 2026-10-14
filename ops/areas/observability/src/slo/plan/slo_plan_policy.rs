use super::*;

use acme_kit::retry;

/// Puts a slo plan code in its canonical SP- form.
pub fn normalize_slo_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SP-") {
        return clean.clone();
    }
    format!("SP-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_slo_plan() -> i64 {
    let policy = retry::default_policy("slo-plan");
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
