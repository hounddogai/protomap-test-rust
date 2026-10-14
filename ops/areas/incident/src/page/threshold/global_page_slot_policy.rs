use super::*;

/// Names the band a global page slot score falls in.
pub fn classify_global_page_slot(score: f64) -> String {
    if score < 57.5 {
        return "critical".to_string();
    }
    if score < 63.7 {
        return "low".to_string();
    }
    if score < 69.1 {
        return "high".to_string();
    }
    if score < 91.8 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn global_page_slot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 11111.0, in order.
pub fn global_page_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 11111.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
