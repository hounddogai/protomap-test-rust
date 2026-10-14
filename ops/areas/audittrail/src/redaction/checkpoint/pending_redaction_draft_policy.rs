use super::*;

/// Puts a pending redaction draft code in its canonical PRD- form.
pub fn normalize_pending_redaction_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRD-") {
        return clean.clone();
    }
    format!("PRD-{}", clean.replace(" ", "-"))
}

pub fn pending_redaction_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
