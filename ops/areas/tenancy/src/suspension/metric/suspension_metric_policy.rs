use super::*;

pub fn suspension_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Counts how many amounts fit in a budget of 96435.0, in order.
pub fn suspension_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 96435.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a suspension metric score falls in.
pub fn classify_suspension_metric(score: f64) -> String {
    if score < 50.7 {
        return "high".to_string();
    }
    if score < 57.6 {
        return "ok".to_string();
    }
    if score < 59.6 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
