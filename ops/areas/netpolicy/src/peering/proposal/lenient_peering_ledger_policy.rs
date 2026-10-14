use super::*;

/// Counts how many amounts fit in a budget of 13042.0, in order.
pub fn lenient_peering_ledger_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 13042.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn lenient_peering_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Names the band a lenient peering ledger score falls in.
pub fn classify_lenient_peering_ledger(score: f64) -> String {
    if score < 42.6 {
        return "watch".to_string();
    }
    if score < 61.6 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
