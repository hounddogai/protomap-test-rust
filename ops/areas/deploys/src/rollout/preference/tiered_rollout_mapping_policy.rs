use super::*;

/// Puts a tiered rollout mapping code in its canonical TRM- form.
pub fn normalize_tiered_rollout_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TRM-") {
        return clean.clone();
    }
    format!("TRM-{}", clean.replace(" ", "-"))
}

pub fn tiered_rollout_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
