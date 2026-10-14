use super::*;

/// Puts a scheduled sink plan code in its canonical SSP- form.
pub fn normalize_scheduled_sink_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SSP-") {
        return clean.clone();
    }
    format!("SSP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 85079.0, in order.
pub fn scheduled_sink_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85079.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn scheduled_sink_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}
