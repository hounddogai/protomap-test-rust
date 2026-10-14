use super::*;

use acme_kit::retry;

/// Puts a global isolation segment code in its canonical GIS- form.
pub fn normalize_global_isolation_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GIS-") {
        return clean.clone();
    }
    format!("GIS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_isolation_segment() -> i64 {
    let policy = retry::default_policy("global-isolation-segment");
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
