use super::*;

pub fn manual_postmortem_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Puts a manual postmortem profile code in its canonical MPP- form.
pub fn normalize_manual_postmortem_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MPP-") {
        return clean.clone();
    }
    format!("MPP-{}", clean.replace(" ", "-"))
}
