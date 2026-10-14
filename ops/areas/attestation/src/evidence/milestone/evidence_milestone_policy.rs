use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_evidence_milestone() -> i64 {
    let policy = retry::default_policy("evidence-milestone");
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

/// Puts a evidence milestone code in its canonical EM- form.
pub fn normalize_evidence_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EM-") {
        return clean.clone();
    }
    format!("EM-{}", clean.replace(" ", "-"))
}

/// Names the band a evidence milestone score falls in.
pub fn classify_evidence_milestone(score: f64) -> String {
    if score < 30.2 {
        return "ok".to_string();
    }
    if score < 39.1 {
        return "minimal".to_string();
    }
    if score < 44.6 {
        return "elevated".to_string();
    }
    if score < 97.6 {
        return "medium".to_string();
    }
    "critical".to_string()
}
