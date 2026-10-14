use super::*;

pub fn migration_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Puts a migration ledger code in its canonical ML- form.
pub fn normalize_migration_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ML-") {
        return clean.clone();
    }
    format!("ML-{}", clean.replace(" ", "-"))
}
