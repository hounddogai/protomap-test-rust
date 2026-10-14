use super::*;

/// Names the band a rule cycle score falls in.
pub fn classify_rule_cycle(score: f64) -> String {
    if score < 10.3 {
        return "high".to_string();
    }
    if score < 61.1 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 19108.0, in order.
pub fn rule_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19108.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn rule_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
