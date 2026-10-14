use super::*;

pub fn flow_log_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Puts a flow log agreement code in its canonical FLA- form.
pub fn normalize_flow_log_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLA-") {
        return clean.clone();
    }
    format!("FLA-{}", clean.replace(" ", "-"))
}
