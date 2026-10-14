use super::*;

/// Puts a batched grant group code in its canonical BGG- form.
pub fn normalize_batched_grant_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BGG-") {
        return clean.clone();
    }
    format!("BGG-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 59797.0, in order.
pub fn batched_grant_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 59797.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn batched_grant_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
