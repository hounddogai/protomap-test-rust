use super::*;

/// Puts a default dashboard metric code in its canonical DDM- form.
pub fn normalize_default_dashboard_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DDM-") {
        return clean.clone();
    }
    format!("DDM-{}", clean.replace(" ", "-"))
}

pub fn default_dashboard_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 77867.0, in order.
pub fn default_dashboard_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 77867.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
