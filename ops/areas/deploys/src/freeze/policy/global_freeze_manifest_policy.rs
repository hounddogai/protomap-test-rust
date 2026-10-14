use super::*;

pub fn global_freeze_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 84061.0, in order.
pub fn global_freeze_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84061.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a global freeze manifest score falls in.
pub fn classify_global_freeze_manifest(score: f64) -> String {
    if score < 24.3 {
        return "medium".to_string();
    }
    if score < 35.3 {
        return "low".to_string();
    }
    if score < 59.3 {
        return "minimal".to_string();
    }
    if score < 92.5 {
        return "high".to_string();
    }
    "watch".to_string()
}
