use super::*;

/// Puts a rolling page audit code in its canonical RPA- form.
pub fn normalize_rolling_page_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPA-") {
        return clean.clone();
    }
    format!("RPA-{}", clean.replace(" ", "-"))
}

pub fn rolling_page_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 5864.0, in order.
pub fn rolling_page_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 5864.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
