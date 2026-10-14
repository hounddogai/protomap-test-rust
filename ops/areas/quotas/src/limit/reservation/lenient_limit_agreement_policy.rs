use super::*;

/// Puts a lenient limit agreement code in its canonical LLA- form.
pub fn normalize_lenient_limit_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LLA-") {
        return clean.clone();
    }
    format!("LLA-{}", clean.replace(" ", "-"))
}

pub fn lenient_limit_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
