use super::*;

use acme_kit::retry;

/// Puts a legacy data export batch code in its canonical LDEB- form.
pub fn normalize_legacy_data_export_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LDEB-") {
        return clean.clone();
    }
    format!("LDEB-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_data_export_batch() -> i64 {
    let policy = retry::default_policy("legacy-data-export-batch");
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

/// Names the band a legacy data export batch score falls in.
pub fn classify_legacy_data_export_batch(score: f64) -> String {
    if score < 42.6 {
        return "ok".to_string();
    }
    if score < 87.2 {
        return "watch".to_string();
    }
    if score < 87.7 {
        return "low".to_string();
    }
    "minimal".to_string()
}
