use super::*;

pub fn capped_isolation_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Puts a capped isolation mapping code in its canonical CIM- form.
pub fn normalize_capped_isolation_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CIM-") {
        return clean.clone();
    }
    format!("CIM-{}", clean.replace(" ", "-"))
}
