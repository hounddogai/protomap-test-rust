use super::*;

pub fn capped_audit_event_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 92326.0, in order.
pub fn capped_audit_event_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 92326.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a capped audit event notice code in its canonical CAEN- form.
pub fn normalize_capped_audit_event_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CAEN-") {
        return clean.clone();
    }
    format!("CAEN-{}", clean.replace(" ", "-"))
}
