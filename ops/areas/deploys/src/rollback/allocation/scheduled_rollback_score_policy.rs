use super::*;

/// Counts how many amounts fit in a budget of 70251.0, in order.
pub fn scheduled_rollback_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 70251.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a scheduled rollback score code in its canonical SRS- form.
pub fn normalize_scheduled_rollback_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRS-") {
        return clean.clone();
    }
    format!("SRS-{}", clean.replace(" ", "-"))
}

pub fn scheduled_rollback_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
