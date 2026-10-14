use super::*;

/// Puts a escalation metric code in its canonical EM- form.
pub fn normalize_escalation_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EM-") {
        return clean.clone();
    }
    format!("EM-{}", clean.replace(" ", "-"))
}

pub fn escalation_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
