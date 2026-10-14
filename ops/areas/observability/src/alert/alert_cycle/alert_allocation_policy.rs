use super::*;

/// Counts how many amounts fit in a budget of 71940.0, in order.
pub fn alert_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 71940.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn alert_allocation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a alert allocation code in its canonical AA- form.
pub fn normalize_alert_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AA-") {
        return clean.clone();
    }
    format!("AA-{}", clean.replace(" ", "-"))
}
