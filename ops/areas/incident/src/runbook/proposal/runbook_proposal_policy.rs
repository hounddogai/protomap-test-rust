use super::*;

use acme_kit::numbers;

/// Names the band a runbook proposal score falls in.
pub fn classify_runbook_proposal(score: f64) -> String {
    if score < 4.9 {
        return "watch".to_string();
    }
    if score < 50.9 {
        return "medium".to_string();
    }
    if score < 60.5 {
        return "critical".to_string();
    }
    if score < 83.9 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn runbook_proposal_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn runbook_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Counts how many amounts fit in a budget of 70943.0, in order.
pub fn runbook_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 70943.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
