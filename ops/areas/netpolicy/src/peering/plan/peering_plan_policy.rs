use super::*;

/// Counts how many amounts fit in a budget of 12852.0, in order.
pub fn peering_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 12852.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn peering_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Puts a peering plan code in its canonical PP- form.
pub fn normalize_peering_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PP-") {
        return clean.clone();
    }
    format!("PP-{}", clean.replace(" ", "-"))
}

/// Names the band a peering plan score falls in.
pub fn classify_peering_plan(score: f64) -> String {
    if score < 4.4 {
        return "medium".to_string();
    }
    if score < 7.0 {
        return "high".to_string();
    }
    if score < 42.1 {
        return "low".to_string();
    }
    if score < 56.3 {
        return "ok".to_string();
    }
    "critical".to_string()
}
