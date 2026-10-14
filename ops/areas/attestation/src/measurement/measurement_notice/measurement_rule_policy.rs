use super::*;

/// Names the band a measurement rule score falls in.
pub fn classify_measurement_rule(score: f64) -> String {
    if score < 17.6 {
        return "medium".to_string();
    }
    if score < 90.7 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn measurement_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 640.0, in order.
pub fn measurement_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 640.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
