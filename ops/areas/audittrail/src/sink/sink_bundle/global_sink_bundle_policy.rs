use super::*;

pub fn global_sink_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Puts a global sink bundle code in its canonical GSB- form.
pub fn normalize_global_sink_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GSB-") {
        return clean.clone();
    }
    format!("GSB-{}", clean.replace(" ", "-"))
}
