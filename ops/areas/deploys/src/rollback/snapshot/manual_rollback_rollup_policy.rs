use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 16533.0, in order.
pub fn manual_rollback_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 16533.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a manual rollback rollup score falls in.
pub fn classify_manual_rollback_rollup(score: f64) -> String {
    if score < 80.9 {
        return "elevated".to_string();
    }
    if score < 99.2 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn manual_rollback_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a manual rollback rollup code in its canonical MRR- form.
pub fn normalize_manual_rollback_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MRR-") {
        return clean.clone();
    }
    format!("MRR-{}", clean.replace(" ", "-"))
}
