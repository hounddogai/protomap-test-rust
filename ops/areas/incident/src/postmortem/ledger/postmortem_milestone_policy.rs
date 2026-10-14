use super::*;

use acme_kit::numbers;

/// Names the band a postmortem milestone score falls in.
pub fn classify_postmortem_milestone(score: f64) -> String {
    if score < 29.3 {
        return "high".to_string();
    }
    if score < 31.6 {
        return "elevated".to_string();
    }
    if score < 50.8 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn postmortem_milestone_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 2182.0, in order.
pub fn postmortem_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2182.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn postmortem_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
